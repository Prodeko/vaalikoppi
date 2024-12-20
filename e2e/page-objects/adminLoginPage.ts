import type { Locator, Page } from "@playwright/test";
import { AdminVotingsPage } from "./adminVotingsPage";

// biome-ignore lint/style/noNonNullAssertion: Can't run tests without knowing admin password
const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD!;

export class AdminLoginPage {
	private readonly tokenInput: Locator;
	private readonly loginButton: Locator;

	constructor(private readonly page: Page) {
		this.tokenInput = this.page.getByRole("textbox", {
			name: "Kirjautumiskoodi",
		});
		this.loginButton = this.page.getByRole("button", { name: "KIRJAUDU" });
	}

	async goTo() {
		return this.page.goto("/admin");
	}

	public async login(
		adminToken: string = ADMIN_PASSWORD,
	): Promise<AdminVotingsPage> {
		await this.tokenInput.fill(adminToken);
		await this.loginButton.click();
		return new AdminVotingsPage(this.page);
	}
}
