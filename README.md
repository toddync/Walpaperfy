# Walpaperfy

**Walpaperfy** is a simple tool that synchronizes your desktop wallpaper and terminal colorscheme with the album art of the currently playing track on Spotify.

### Prerequisites

-   **pywal**: Used to set the background image and generate the colorscheme.
-   **Wal Theme** VsCode extension (optional): uses the colorscheme on vscode.

### Setup

1. **Register a Spotify App**:
   [Register a new application on Spotify's developer portal](https://developer.spotify.com/dashboard/applications) to obtain your **Client ID** and **Client Secret**.
   Add `http://localhost:8080/` as one of the redirect URIs for your app in the Spotify dashboard, and include the credentials in your `env.rs` file.
2. **Get the Refresh Token**:
   To retrieve your refresh token:

    - Run the `getToken.js` script.
    - Open the URL that appears in your terminal.
    - After being redirected, copy the code from the URL (the part after `code=`) and paste it back into the terminal. You'll then receive the refresh token to add to your `env.rs` file.

    ```bash
    node getToken.js
    ```

    The more api's you set, the better, the code will switch between them to avoid the 13h wait time once you get rate-limited. To set more api's, just repeat the steps in this readme, up to this one.

3. **Build the Application**:
   After setting everything up, run the following command to start the synchronization:

    ```bash
    cargo build --release
    ```

    The final binary should be on the `target/release` folder, just run it.

    ```bash
    ./target/release/walpaperfy
    ```

Your wallpaper will now automatically update with the album art of the song you're currently listening to on Spotify!
