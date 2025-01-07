# Walpaperfy

**Walpaperfy** is a simple tool that synchronizes your desktop wallpaper and terminal colorscheme with the album art of the currently playing track on Spotify.

### Prerequisites

-   **pywal**: Used to set the background image and generate the colorscheme.
-   **Wal Theme** VsCode extension (optional): uses the colorscheme on vscode.

### Setup

1. **Install the applicatin**
   Just use cargo.

    ```bash
     cargo install walpaperfy
    ```

    Or clone the repo and build it.

    ```bash
    g4it clone https://github.com/toddync/Walpaperfy
    cd walpaperfy
    cargo build --release && cargo install --path .
    ```

2. **Register a Spotify App**:
   [Register a new application on Spotify's developer portal](https://developer.spotify.com/dashboard/applications) to obtain your **Client ID** and **Client Secret**.
   Add `http://localhost` as one of the redirect URIs for your app in the Spotify dashboard.
3. **Get the Refresh Token**:
   To retrieve your refresh token:

    - Run the application with the `--add-key` tag, with your credentials from step 2.
    - Open the URL that appears in your terminal.
    - After being redirected, copy the code from the URL (the part after `code=`) and paste it back into the terminal. You'll then receive the refresh token to add to your `env.rs` file.

    ```bash
    walpaperfy --add-key
    ```

    The more api's you set, the better, but it runs smoothtly with only 3 for me. The code will switch between them to avoid the 13h wait time once you get rate-limited. To set more api's, just repeat the steps above.

4. **Run the Application**:

    ```bash
     walpaperfy
    ```

    Your wallpaper will now automatically update with the album art of the song you're currently listening to on Spotify!
