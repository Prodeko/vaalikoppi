import type { Locator, Page } from "@playwright/test";
import { CreateVotingBox } from "../components/createVotingBox";
import { AdminNavBar } from "../components/adminNavBar";

export class AdminHomePage {
	public readonly navBar: AdminNavBar;
	private readonly createVotingBox: CreateVotingBox;

	constructor(private readonly page: Page) {
		this.navBar = new AdminNavBar(this.page.getByRole("navigation"));
		this.createVotingBox = new CreateVotingBox(this.page);
	}

	public async expectIsVisible() {
		await this.createVotingBox.expectIsVisible();
	}
}
