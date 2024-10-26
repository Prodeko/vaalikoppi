import { expect, type Locator, type Page } from "@playwright/test";
import { AdminNavBar } from "../components/adminNavBar";
import { AdminVotingsPage } from "./adminVotingsPage";

export class TokensPage {
	public readonly navBar: AdminNavBar;
	private readonly generateBulkTokensButton: Locator;
	private readonly votingsLink: Locator;
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

	public async generateBulkTokens(): Promise<string[]> {
		const response = this.page.waitForResponse(/.*tokens.*/);

		const [responseBody, _] = await Promise.all([
			response,
			this.generateBulkTokensButton.click(),
		]);

		const responseBodyWords = new Set(
			(await responseBody.text()).split(/<\/?td>/),
		);

		const allTokenRows = (await this.tokenRows.all()).slice(-100);

		const allTokens = (
			await Promise.all(
				allTokenRows.map((row) => row.getByRole("cell").first().textContent()),
			)
		)
			.filter((s) => s != null)
			.filter((token) => responseBodyWords.has(token));

		return allTokens;
	}

	public async goToPrint() {
		await this.printTokensLink.click();
	}

	public async goToVotings(): Promise<AdminVotingsPage> {
		await this.navBar.normalNavBar.goToHome();
		return new AdminVotingsPage(this.page);
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

	public async activateToken(token: string) {
		const tokenRow = this.tokenRows.filter({ hasText: token });
		await tokenRow.getByRole("button", { name: "Aktivoi" }).click();

		const response = this.page.waitForResponse(/.*\/tokens.*/);
		const click = tokenRow.getByRole("button", { name: "Aktivoi?" }).click();

		await Promise.all([response, click]);
	}

	public async voidToken(token: string) {
		const tokenRow = this.tokenRows.filter({ hasText: token });
		await tokenRow.getByRole("button", { name: "Mitätöi" }).click();

		const response = this.page.waitForResponse(/.*\/tokens.*/);
		const click = tokenRow.getByRole("button", { name: "Mitätöi?" }).click();

		await Promise.all([response, click]);
	}
}
