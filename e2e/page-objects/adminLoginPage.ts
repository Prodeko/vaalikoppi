import type { Locator, Page } from "@playwright/test";
import { NavBarPage } from "./navBarPage";
import type { VoterLoginDetails } from "../types";

export class AdminLoginPage extends NavBarPage {
	private readonly tokenInput: Locator;
	private readonly loginButton: Locator;

	constructor(private readonly page: Page) {
		super(page);

		this.tokenInput = this.page.getByRole("textbox", {
			name: "Kirjautumiskoodi",
		});
		this.loginButton = this.page.getByRole("button", { name: "KIRJAUDU" });
	}

	async goTo() {
		return this.page.goto("/admin");
	}

	public async login({ alias, token }: VoterLoginDetails) {
		await this.tokenInput.fill(token);
		await this.loginButton.click();
	}
}
