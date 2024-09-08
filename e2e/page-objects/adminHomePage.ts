import type { Page } from "@playwright/test";
import { NavBarPage } from "./navBarPage";
import { CreateVotingBox } from "../components/createVotingBox";

export class AdminHomePage extends NavBarPage {
	private readonly createVotingBox: CreateVotingBox;

	constructor(private readonly page: Page) {
		super(page);
		this.createVotingBox = new CreateVotingBox(this.page);
	}

	public async expectIsVisible() {
		await this.createVotingBox.expectIsVisible();
	}
}
