import type { Locator, Page } from "@playwright/test";
import { NavBarPage } from "./navBarPage";

export class LoginPage extends NavBarPage {
	private readonly tokenInput: Locator;
	private readonly loginButton: Locator;

	constructor(private readonly page: Page) {
		super(page);
		this.tokenInput = this.page.getByRole("textbox", {
			name: "Kirjautumiskoodi",
		});
		this.loginButton = this.page.getByRole("button", { name: "KIRJAUDU" });
	}

	public async login(adminToken: string) {
		await this.tokenInput.fill(adminToken);
		await this.loginButton.click();
	}
}
