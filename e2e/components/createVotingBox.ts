import { expect, type Locator, type Page } from "@playwright/test";

export class CreateVotingBox {
	private readonly locator: Locator;
	private readonly hideVoteCountCheckbox: Locator;
	private readonly title: Locator;

	constructor(private readonly page: Page) {
		const title = this.page.getByText("Luo uusi äänestys");
		const hideVoteCountCheckbox = this.page.getByRole("checkbox", {
			name: "Piilota äänten määrä",
		});

		this.locator = this.page
			.locator("div")
			.filter({ has: title })
			.filter({ has: hideVoteCountCheckbox });

		this.title = this.locator.locator(title);
		this.hideVoteCountCheckbox = this.locator.locator(hideVoteCountCheckbox);
	}

	public async expectIsVisible() {
		return Promise.all([
			expect(this.title).toBeVisible(),
			expect(this.hideVoteCountCheckbox).toBeVisible(),
		]);
	}
}
