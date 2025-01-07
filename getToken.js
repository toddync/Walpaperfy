const readline = require("readline");

const rl = readline.createInterface({
	input: process.stdin,
	output: process.stdout,
});

const ID = "YOUR_SPOTIFY_CLIENT_ID";
const SECRET = "YOUR_SPOTIFY_CLIENT_SECRET";

(async () => {
	let scope = "user-read-currently-playing"; // Request permission to read the currently playing song
	let red = "http://localhost"; // Redirect URI, this needs to be registered in the Spotify developer dashboard

	// Display a message to the user to open the authorization URL in their browser
	console.log(
		`go to this link:\n https://accounts.spotify.com/authorize?response_type=code&client_id=${ID}&scope=${scope}&redirect_uri=${red} \n`
	);

	rl.question("The code ", async (code) => {
		// Create the parameters for the POST request to exchange the authorization code for a token
		const params = new URLSearchParams();
		params.append("code", code); // The authorization code received from the user
		params.append("redirect_uri", red); // The same redirect URI used in the authorization request
		params.append("grant_type", "authorization_code"); // Specify the grant type as authorization code

		// Make a POST request to Spotify's token endpoint to exchange the authorization code for an access token
		const response = await fetch("https://accounts.spotify.com/api/token", {
			method: "POST",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded",
				Authorization:
					"Basic " +
					Buffer.from(`${ID}:${SECRET}`).toString("base64"),
			},
			body: params,
		});

		if (!response.ok) {
			console.error("Error:", response.status, await response.text());
		} else {
			const data = await response.json();
			console.log("\nyour token:", data.refresh_token);
		}

		rl.close();
	});
})();
