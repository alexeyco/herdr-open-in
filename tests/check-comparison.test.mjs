import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createServer } from "node:http";
import { once } from "node:events";
import test from "node:test";

const checker = new URL("../scripts/check-comparison.mjs", import.meta.url);
const repository = "example/repo";
const releaseCell = "github.com/example/repo/releases/tag/";

async function withFixture(handler, callback) {
  const directory = mkdtempSync(join(tmpdir(), "comparison-check-"));
  const server = createServer(handler);
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  const api = `http://127.0.0.1:${address.port}`;
  const readme = join(directory, "README.md");
  const workflow = join(directory, "comparison-freshness.yml");
  const report = join(directory, "drift-report.md");

  try {
    await callback({ directory, api, readme, workflow, report });
  } finally {
    server.close();
    server.closeAllConnections();
    await once(server, "close");
    rmSync(directory, { recursive: true, force: true });
  }
}

function reply(response, status, body = "") {
  response.writeHead(status, { "Content-Type": "application/json" });
  response.end(body);
}

function fixtureEnv(api, extra = {}) {
  return {
    ...process.env,
    EXPECT_REPOS: `${repository}@v1.0.0`,
    COMPARISON_GITHUB_API: api,
    GITHUB_TOKEN: "test-token",
    GITHUB_REPOSITORY: "owner/project",
    ...extra,
  };
}

function invoke(args, env) {
  const child = spawn(process.execPath, [checker.pathname, ...args], { env });
  const stdout = [];
  const stderr = [];
  child.stdout.on("data", (chunk) => stdout.push(chunk));
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => child.kill("SIGKILL"), 30_000);
    child.once("error", reject);
    child.once("close", (status, signal) => {
      clearTimeout(timeout);
      resolve({
        status,
        signal,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      });
    });
  });
}

function readmeFixture(api, release = "v1.0.0", linkPath = "/live") {
  return [
    "# Fixture",
    "",
    "## Comparison",
    "",
    `Latest release: ${releaseCell}${release}`,
    `Source: ${api}${linkPath}`,
    "",
    "## Following section",
    "",
  ].join("\n");
}

test("clean comparison succeeds without changing files or reporting drift", async () => {
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 200, JSON.stringify({ tag_name: "v1.0.0" }));
    } else if (request.url === "/live" && request.method === "HEAD") {
      reply(response, 204);
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, report }) => {
    const original = readmeFixture(api);
    writeFileSync(readme, original);

    const result = await invoke(
      ["--readme", readme, "--report", report],
      fixtureEnv(api),
    );

    assert.equal(result.status, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /comparison-check: ok/);
    assert.equal(readFileSync(readme, "utf8"), original);
    assert.throws(() => readFileSync(report), { code: "ENOENT" });
  });
});

test("release drift is fixed in the README and workflow expectation", async () => {
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 200, JSON.stringify({ tag_name: "v2.0.0" }));
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, workflow }) => {
    writeFileSync(
      readme,
      `## Comparison\n\n[ v1.0.0 ](https://github.com/${repository}/releases/tag/v1.0.0)\n`,
    );
    writeFileSync(
      workflow,
      `env:\n  EXPECT_REPOS: ${repository}@v1.0.0\n`,
    );

    const result = await invoke(
      ["--fix", "--readme", readme, "--workflow", workflow],
      fixtureEnv(api),
    );

    assert.equal(result.status, 0, result.stderr || result.stdout);
    assert.match(readFileSync(readme, "utf8"), /releases\/tag\/v2\.0\.0/);
    assert.match(readFileSync(workflow, "utf8"), /EXPECT_REPOS: example\/repo@v2\.0\.0/);
  });
});

test("dead links create an issue report and issue mode creates one issue", async () => {
  const issueRequests = [];
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 200, JSON.stringify({ tag_name: "v1.0.0" }));
    } else if (request.url === "/dead" && request.method === "HEAD") {
      reply(response, 404);
    } else if (
      request.url === "/repos/owner/project/issues?labels=comparison-drift&state=open" &&
      request.method === "GET"
    ) {
      issueRequests.push("search");
      reply(response, 200, "[]");
    } else if (
      request.url === "/repos/owner/project/labels" &&
      request.method === "POST"
    ) {
      issueRequests.push("label");
      reply(response, 422, JSON.stringify({ message: "already exists" }));
    } else if (
      request.url === "/repos/owner/project/issues" &&
      request.method === "POST"
    ) {
      issueRequests.push("create");
      reply(response, 201, JSON.stringify({ number: 7 }));
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, report }) => {
    writeFileSync(readme, readmeFixture(api, "v1.0.0", "/dead"));

    const check = await invoke(
      ["--readme", readme, "--report", report],
      fixtureEnv(api),
    );
    assert.equal(check.status, 1, check.stderr || check.stdout);
    assert.match(readFileSync(report, "utf8"), /dead-link/);

    const issue = await invoke(["--report-issue", report], fixtureEnv(api));
    assert.equal(issue.status, 0, issue.stderr || issue.stdout);
    assert.match(issue.stdout, /created issue #7/);
    assert.deepEqual(issueRequests, ["search", "label", "create"]);
  });
});

test("moved links create a report and update an existing issue", async () => {
  const issueRequests = [];
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 200, JSON.stringify({ tag_name: "v1.0.0" }));
    } else if (request.url === "/old" && request.method === "HEAD") {
      response.writeHead(302, { Location: "/new" });
      response.end();
    } else if (request.url === "/new" && request.method === "HEAD") {
      reply(response, 204);
    } else if (
      request.url === "/repos/owner/project/issues?labels=comparison-drift&state=open" &&
      request.method === "GET"
    ) {
      issueRequests.push("search");
      reply(response, 200, JSON.stringify([{ number: 19 }]));
    } else if (
      request.url === "/repos/owner/project/issues/19" &&
      request.method === "PATCH"
    ) {
      issueRequests.push("update");
      reply(response, 200, JSON.stringify({ number: 19 }));
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, report }) => {
    writeFileSync(readme, readmeFixture(api, "v1.0.0", "/old"));

    const check = await invoke(
      ["--readme", readme, "--report", report],
      fixtureEnv(api),
    );
    assert.equal(check.status, 1, check.stderr || check.stdout);
    assert.match(readFileSync(report, "utf8"), /moved-url/);
    assert.match(readFileSync(report, "utf8"), new RegExp(`${api}/new`));

    const issue = await invoke(["--report-issue", report], fixtureEnv(api));
    assert.equal(issue.status, 0, issue.stderr || issue.stdout);
    assert.match(issue.stdout, /updated issue #19/);
    assert.deepEqual(issueRequests, ["search", "update"]);
  });
});

test("a missing release cell is reported without rewriting the README", async () => {
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 200, JSON.stringify({ tag_name: "v1.0.0" }));
    } else if (request.url === "/live" && request.method === "HEAD") {
      reply(response, 204);
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, report }) => {
    const original = `## Comparison\n\nSource: ${api}/live\n`;
    writeFileSync(readme, original);

    const result = await invoke(
      ["--readme", readme, "--report", report],
      fixtureEnv(api),
    );

    assert.equal(result.status, 1, result.stderr || result.stdout);
    assert.match(readFileSync(report, "utf8"), /missing-cell/);
    assert.equal(readFileSync(readme, "utf8"), original);
  });
});

test("unavailable source fails without mutating files or writing an issue report", async () => {
  let issueRequests = 0;
  await withFixture((request, response) => {
    if (request.url === "/repos/example/repo/releases/latest") {
      reply(response, 403);
    } else if (request.url?.startsWith("/repos/owner/project/issues")) {
      issueRequests += 1;
      reply(response, 200, "[]");
    } else if (request.url === "/live" && request.method === "HEAD") {
      reply(response, 204);
    } else {
      reply(response, 404);
    }
  }, async ({ api, readme, report }) => {
    const original = readmeFixture(api);
    writeFileSync(readme, original);

    const result = await invoke(
      ["--readme", readme, "--report", report],
      fixtureEnv(api),
    );

    assert.equal(result.status, 2, result.stderr || result.stdout);
    assert.match(result.stdout, /unavailable/);
    assert.equal(readFileSync(readme, "utf8"), original);
    assert.throws(() => readFileSync(report), { code: "ENOENT" });
    assert.equal(issueRequests, 0);
  });
});
