import { expect, type Locator, type Page } from "@playwright/test";
import { CreateVotingBox } from "../components/createVotingBox";
import { AdminNavBar } from "../components/adminNavBar";
import { TokensPage } from "./tokensPage";
import type { Locatable, CreateVoting } from "../types";

export class AdminVotingsPage {
	private readonly navBar: AdminNavBar;
	private readonly createVotingBox: CreateVotingBox;
	private readonly votings: Locator;

	constructor(private readonly page: Page) {
		this.navBar = new AdminNavBar(this.page.getByRole("navigation"));
		this.createVotingBox = new CreateVotingBox(this.page);
		this.votings = this.page.getByTestId(/voting-.*/);
	}

	public async expectIsVisible() {
		await this.createVotingBox.expectIsVisible();
	}

	public async goToTokens(): Promise<TokensPage> {
		await this.navBar.tokensLink.click();
		return new TokensPage(this.page);
	}

	public async expectVotingExists(voting: Locatable<CreateVoting>) {
		await expect(this.votings.getByText(voting.name ?? "")).toBeVisible();
	}

	public async createVoting(voting: CreateVoting) {
		await this.createVotingBox.create(voting);
		const draftVoting = this.votings.filter({ hasText: voting.name });
		const nameField = draftVoting.getByRole("textbox");
		const addButton = draftVoting.getByRole("button", { name: "Lisää" });
		const openButton = draftVoting.getByRole("button", { name: "Avaa" });

		for (const candidate of voting.candidates) {
			await nameField.fill(candidate);
			await addButton.click();
		}

		await openButton.click();
	}
}
