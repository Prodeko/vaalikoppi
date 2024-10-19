import { defineConfig, devices } from "@playwright/test";
import dotenv from "dotenv";
import path from "node:path";

dotenv.config({ path: path.resolve(__dirname, ".env") });

// biome-ignore lint/style/noNonNullAssertion: <explanation>
const port = process.env.PORT!;

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
	testDir: "./e2e",
	/* Run tests in files in parallel */
	fullyParallel: true,
	/* Fail the build on CI if you accidentally left test.only in the source code. */
	forbidOnly: !!process.env.CI,
	/* Retry on CI only */
	retries: process.env.CI ? 2 : 0,
	/* Opt out of parallel tests on CI. */
	workers: process.env.CI ? undefined : undefined,
	/* Reporter to use. See https://playwright.dev/docs/test-reporters */
	reporter: "html",
	/* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
	use: {
		baseURL: `http://127.0.0.1:${port}`,
		/* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
		trace: "on-first-retry",
	},

	/* Configure projects for major browsers */
	projects: [
		{
			name: "chromium",
			use: { ...devices["Desktop Chrome"] },
		},

		{
			name: "firefox",
			use: { ...devices["Desktop Firefox"] },
		},
		//		{
		//			name: "webkit",
		//			use: { ...devices["Desktop Safari"] },
		//		},
		//		{
		//		name: "Mobile Chrome",
		//		use: { ...devices["Pixel 5"] },
		//		},
		//		{
		//			name: "Mobile Safari",
		//			use: { ...devices["iPhone 12"] },
		//		},
	],

	/* Run your local dev server before starting the tests */
	webServer: process.env.CI
		? undefined
		: {
				command: "sqlx migrate revert && sqlx migrate run && cargo run",
				url: `http://localhost:${port}`,
			},
});
