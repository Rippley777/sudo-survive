import { test, expect } from "@playwright/test";

test("buy, deploy, investigate, contain, recover, and retain state across reload", async ({
  page,
  request,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await expect(
    page.getByRole("button", { name: "Inspect WEB-01" }),
  ).toBeVisible();
  const initial = await page.getByTestId("balance").innerText();
  await expect
    .poll(() => page.getByTestId("balance").innerText())
    .not.toBe(initial);
  await page.getByRole("button", { name: "Add server" }).click();
  await page.getByRole("button", { name: "Buy server $90" }).click();
  await expect(page.getByLabel("Selected asset")).toHaveValue("WEB-02");
  await page.getByRole("button", { name: "Deploy service" }).click();
  await expect(page.locator("footer")).toContainText("2/2 services online");
  await expect(
    page.getByRole("button", { name: "Investigate", exact: true }),
  ).toBeVisible({ timeout: 25_000 });
  await page.getByRole("button", { name: "Pause simulation" }).click();
  await page.getByRole("button", { name: "Investigate", exact: true }).click();
  await expect(page.getByRole("table")).toContainText("AUTH_LOGIN_FAILED");
  await page
    .getByLabel("Search security logs")
    .fill("event = AUTH_LOGIN_FAILED AND user = deploy");
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.locator(".event-kind").first()).toHaveText(
    "AUTH_LOGIN_FAILED",
  );
  await page.screenshot({ path: "test-results/investigation.png" });
  await page.getByRole("button", { name: "Response", exact: true }).click();
  await page.getByRole("button", { name: "Block source", exact: true }).click();
  await expect(page.locator(".report-heading")).toContainText("Contained");
  await expect(page.locator(".report-analysis")).toContainText(
    "No access established",
  );

  await page.getByRole("button", { name: "Field guide" }).click();
  await page
    .getByLabel("Practice incident type")
    .selectOption("stolen_credential");
  await page.getByRole("button", { name: "Run exercise" }).click();
  await page.getByRole("button", { name: "Resume simulation" }).click();
  await expect(
    page.getByRole("button", { name: "Investigate", exact: true }),
  ).toBeVisible({ timeout: 15_000 });
  await page.getByRole("button", { name: "Pause simulation" }).click();
  await page.getByRole("button", { name: "Investigate", exact: true }).click();
  await expect(page.getByRole("table")).toContainText("AUTH_LOGIN_SUCCESS");
  await page.getByRole("button", { name: "Response", exact: true }).click();
  const affected = await page.getByLabel("Selected asset").inputValue();
  await page.getByRole("button", { name: "Block source", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Isolate host" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Isolate host" }).click();
  await expect(page.locator(".report-heading")).toContainText("Contained");
  await expect(page.locator(".report-analysis")).toContainText(affected);
  await page.getByRole("button", { name: "Response", exact: true }).click();
  await page.getByRole("button", { name: "Resume host" }).click();
  await expect(page.getByRole("status")).toContainText(
    "restore a clean backup",
  );
  await page
    .getByRole("button", { name: "Rotate credential", exact: true })
    .click();
  await page.getByRole("button", { name: "Restore · $20" }).click();
  await page.getByRole("button", { name: "Resume simulation" }).click();
  await expect(page.getByRole("button", { name: "Resume host" })).toBeEnabled({
    timeout: 10_000,
  });
  await page.getByRole("button", { name: "Resume host" }).click();
  await expect(page.locator("footer")).toContainText("2/2 services online");
  await page.reload();
  await expect(
    page.getByRole("button", { name: `Inspect ${affected}` }),
  ).toBeVisible();
  const state = await (await request.get("/api/state")).json();
  expect(state.servers).toHaveLength(2);
  expect(state.user.credential.version).toBe(2);
  expect(state.reports).toHaveLength(2);
  expect(state.reports[1].data_lost_mb).toBe(0);
  expect(errors).toEqual([]);
});

test("compact, sidebar, collapsed, and query-error views remain usable", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1100, height: 330 });
  await page.goto("/");
  await expect(
    page.getByRole("button", { name: "Inspect WEB-01" }),
  ).toBeVisible();
  const name = await page.locator(".rack-name").first().boundingBox();
  const heading = await page.locator(".scene-heading").boundingBox();
  expect(name!.y).toBeGreaterThanOrEqual(heading!.y + heading!.height);
  await page.screenshot({ path: "test-results/compact.png" });
  await page.getByRole("button", { name: "Right dock", exact: true }).click();
  await page.setViewportSize({ width: 390, height: 780 });
  await expect(
    page.getByRole("button", { name: "Add server" }),
  ).toBeInViewport();
  await page.screenshot({ path: "test-results/sidebar.png" });
  await page.getByRole("button", { name: "Security console" }).click();
  await page.getByLabel("Search security logs").fill("unsupported = value");
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Unknown field");
  await page.getByRole("button", { name: "Collapse game" }).click();
  await expect(page.locator(".collapsed-content")).toBeVisible();
  await page.screenshot({ path: "test-results/collapsed.png" });
});
