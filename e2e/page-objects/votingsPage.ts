import { expect, type Locator, type Page } from "@playwright/test";
import { NavBar } from "../components/navBar";

export class VotingsPage {
	private readonly navBar: NavBar;
	private readonly loginStatusBox: Locator;

	constructor(private readonly page: Page) {
		this.navBar = new NavBar(this.page.getByRole("navigation"));
		this.loginStatusBox = this.page.getByTestId("login-status-box");
	}

	public async expectIsVisible() {
		await expect(this.loginStatusBox).toBeVisible();
	}
}
