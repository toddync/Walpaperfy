import requests
from base64 import b64encode

client_id = "YOUR_SPOTIFY_CLIENT-ID"
client_secret = "YOUR_SPOTIFY_CLIENT-SECRET"

scope = "user-read-currently-playing"
redirect_uri = "http://localhost:8080/"

print(f"go to:\nhttps://accounts.spotify.com/authorize?response_type=code&client_id={client_id}&scope={scope}&redirect_uri={redirect_uri}")

code = input("\nthe code: ");

data = {
    "code": code,
    "redirect_uri": redirect_uri,
    "grant_type": "authorization_code"
}

auth_header = b64encode(f"{client_id}:{client_secret}".encode()).decode()

response = requests.post(
    "https://accounts.spotify.com/api/token",
    data=data,
    headers={
        "Content-Type": "application/x-www-form-urlencoded",
        "Authorization": f"Basic {auth_header}"
    }
)

if response.status_code == 200:
    token_data = response.json()
    print("\ntoken:", token_data.get("refresh_token"))
else:
    print("Failed to get access token:", response.status_code, response.text)
