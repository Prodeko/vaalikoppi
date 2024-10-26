import { expect, type Locator, type Page } from "@playwright/test";
import type { CreateVoting } from "../types";

export class CreateVotingBox {
	private readonly locator: Locator;
	private readonly title: Locator;
	private readonly hideVoteCountCheckbox: Locator;
	private readonly numberOfSeatsInput: Locator;
	private readonly nameInput: Locator;
	private readonly descriptionInput: Locator;
	private readonly createButton: Locator;

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
		this.numberOfSeatsInput = this.locator.locator(
			this.page.getByRole("spinbutton", { name: "Kuinka monta valitaan?" }),
		);
		this.nameInput = this.locator.locator(
			this.page.getByRole("textbox", { name: "Äänestyksen nimi" }),
		);
		this.descriptionInput = this.locator.locator(
			this.page.getByRole("textbox", { name: "Kuvaus" }),
		);
		this.createButton = this.locator.locator(
			this.page.getByRole("button", { name: "Luo äänestys" }),
		);
	}

	public async expectIsVisible() {
		return Promise.all([
			expect(this.title).toBeVisible(),
			expect(this.hideVoteCountCheckbox).toBeVisible(),
		]);
	}

	public async create(voting: CreateVoting) {
		await this.hideVoteCountCheckbox.setChecked(voting.hideVoteCount);
		await this.numberOfSeatsInput.fill(voting.seats.toString());
		await this.nameInput.fill(voting.name);
		await this.descriptionInput.fill(voting.description);
		await this.createButton.click();
	}
}
