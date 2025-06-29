This project is a web application that converts YouTube videos to MP3 files.

It is built with Rust and the Leptos framework. The frontend is built with Leptos and Tailwind CSS, and the backend is built with Axum. It uses `yt-dlp` to download and convert the YouTube videos.

The application allows a user to paste a YouTube URL into a text input field. When the "Convert Now" button is clicked, the application makes a request to the backend to start the conversion process. The backend then uses `yt-dlp` to download the audio from the YouTube video and convert it to an MP3 file. While the conversion is in progress, the frontend polls the backend to check the status of the conversion. Once the conversion is complete, a "Download MP3" button is displayed, which allows the user to download the converted MP3 file.
