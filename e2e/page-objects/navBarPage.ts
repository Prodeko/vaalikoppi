import type { Page } from "@playwright/test";
import { NavBar } from "../components/navBar";

export class NavBarPage {
	protected navBar: NavBar;

	constructor(page: Page) {
		this.navBar = new NavBar(page.getByRole("navigation"));
	}
}
