import type { Locator } from "@playwright/test";
import { NavBar } from "./navBar";

export class AdminNavBar {
	public readonly normalNavBar: NavBar;
	private readonly votingsLink: Locator;
	private readonly tokensLink: Locator;

	constructor(private readonly locator: Locator) {
		this.normalNavBar = new NavBar(this.locator);
		this.votingsLink = this.locator.getByRole("link", { name: "Äänsetykset" });
		this.tokensLink = this.locator.getByRole("link", { name: "Koodit" });
	}
}
