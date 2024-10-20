import { expect, type Locator, type Page } from "@playwright/test";
import { CreateVotingBox } from "../components/createVotingBox";
import { AdminNavBar } from "../components/adminNavBar";
import { TokensPage } from "./tokensPage";
import type { Locatable, VotingMeta } from "../types";

export class AdminVotingsPage {
	private readonly navBar: AdminNavBar;
	private readonly createVotingBox: CreateVotingBox;

	constructor(private readonly page: Page) {
		this.navBar = new AdminNavBar(this.page.getByRole("navigation"));
		this.createVotingBox = new CreateVotingBox(this.page);
	}

	public async expectIsVisible() {
		await this.createVotingBox.expectIsVisible();
	}

	public async goToTokens(): Promise<TokensPage> {
		await this.navBar.tokensLink.click();
		return new TokensPage(this.page);
	}

	public async expectVotingExists(voting: Locatable<VotingMeta>) {
		await expect(
			this.page.getByTestId(/voting-.*/).getByText(voting.name ?? ""),
		).toBeVisible();
	}

	public async createVoting(voting: VotingMeta) {
		await this.createVotingBox.create(voting);
	}
}
