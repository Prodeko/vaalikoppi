export function generateRandomString(length = 10) {
	return Math.random().toString(20).substring(2, length);
}
