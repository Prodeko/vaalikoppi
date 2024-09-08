import type { Locator, Page } from "@playwright/test";
import type { VoterLoginDetails } from "../types";
import { NavBar } from "../components/navBar";

export class LoginPage {
	private readonly navBar: NavBar;
	private readonly tokenInput: Locator;
	private readonly loginButton: Locator;

	constructor(private readonly page: Page) {
		this.navBar = new NavBar(this.page.getByRole("navigation"));
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
