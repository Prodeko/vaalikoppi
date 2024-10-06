import { expect, type Locator, type Page } from "@playwright/test";
import { AdminNavBar } from "../components/adminNavBar";

export class TokensPage {
	public readonly navBar: AdminNavBar;
	private readonly generateBulkTokensButton: Locator;
	private readonly printTokensLink: Locator;
	private readonly invalidateAllTokensButton: Locator;
	private readonly tokenRows: Locator;

	constructor(public readonly page: Page) {
		this.navBar = new AdminNavBar(this.page.getByRole("navigation"));
		this.generateBulkTokensButton = this.page.getByRole("button", {
			name: /Generoi .* uutta koodia/,
		});
		this.printTokensLink = this.page.getByRole("link", {
			name: "tulosta koodit",
		});
		this.invalidateAllTokensButton = this.page.getByRole("button", {
			name: "Mitätöi aktiiviset koodit",
		});
		this.tokenRows = this.page
			.getByTestId("tokens-table-body")
			.getByRole("row");
	}

	public async generateBulkTokens() {
		await this.generateBulkTokensButton.click();
	}

	public async goToPrint() {
		await this.printTokensLink.click();
	}

	public async invalidateAllTokens() {
		await this.invalidateAllTokensButton.click();
	}

	public async expectIsVisible() {
		await expect(this.generateBulkTokensButton).toBeVisible();
	}

	public async expectCanSeeTokens() {
		await expect(this.tokenRows.first()).toBeVisible();
	}
}
