import { expect, type Locator, type Page } from "@playwright/test";
import { AdminNavBar } from "../components/adminNavBar";

export class TokensPage {
	public readonly navBar: AdminNavBar;
	private readonly generateBulkTokensButton: Locator;
	private readonly printTokensLink: Locator;
	private readonly invalidateAllTokensButton: Locator;
	private readonly tokenRows: Locator;
	private readonly tokensTableBody: Locator;

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
		this.tokensTableBody = this.page.getByTestId("tokens-table-body");
		this.tokenRows = this.tokensTableBody.getByRole("row");
	}

	public async generateBulkTokens(): Promise<string[]> {
		const response = this.page.waitForResponse(/.*tokens.*/);

		await Promise.all([
			response,
			this.generateBulkTokensButton.click(),
		]);

		const allTokenRows = (await this.tokenRows.all()).slice(-100);
		const tokens = await Promise.all(
			allTokenRows.map((row) => row.getAttribute("data-token")),
		);

		return tokens.filter((token): token is string => !!token);
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

	private getTokenRow(token: string): Locator {
		return this.tokensTableBody.locator(`tr[data-token="${token}"]`);
	}

	public async activateToken(token: string) {
		const tokenRow = this.getTokenRow(token);
		await tokenRow.getByRole("button", { name: "Aktivoi" }).click();

		const response = this.page.waitForResponse(/.*\/tokens.*/);
		const click = tokenRow.getByRole("button", { name: "Aktivoi?" }).click();

		await Promise.all([response, click]);
	}

	public async voidToken(token: string) {
		const tokenRow = this.getTokenRow(token);
		await tokenRow.getByRole("button", { name: "Mitätöi" }).click();

		const response = this.page.waitForResponse(/.*\/tokens.*/);
		const click = tokenRow.getByRole("button", { name: "Mitätöi?" }).click();

		await Promise.all([response, click]);
	}
}
