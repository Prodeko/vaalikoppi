import type { Locator, Page } from "@playwright/test";
import { NavBarPage } from "./navBarPage";
import type { VoterLoginDetails } from "../types";

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

	public async goTo() {
		await this.page.goto("");
		await this.navBar.logout();
	}

	public async login({ alias, token }: VoterLoginDetails) {
		await this.tokenInput.fill(token);
		await this.loginButton.click();
	}
}
