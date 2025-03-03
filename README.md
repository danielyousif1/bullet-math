# 🚀 Bullet Math: Speed Math Multiplayer Game

**Bullet Math** is a fast-paced, multiplayer math game inspired by [ZetaMac](https://arithmetic.zetamac.com/). 
<br/>Test your arithmetic skills in real-time against players from around the world!

[![Play Now](https://img.shields.io/badge/Play-Now-brightgreen?style=for-the-badge&logo=heroku)](https://bullet-math-4ed37ad30368.herokuapp.com/)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![HTML](https://img.shields.io/badge/HTML-5-red?style=for-the-badge&logo=html5)](https://developer.mozilla.org/en-US/docs/Web/HTML)
[![CSS](https://img.shields.io/badge/CSS-3-blue?style=for-the-badge&logo=css3)](https://developer.mozilla.org/en-US/docs/Web/CSS)

---

## 🎮 How to Play

1. **Visit the Game**: Click the [Play Now](https://bullet-math-4ed37ad30368.herokuapp.com/) button to start playing.
2. **Solve Math Problems**: Answer as many arithmetic questions as you can within the time limit.
3. **Compete**: Challenge other players in real-time and see who comes out on top!

---

## 🖥️ Screenshot

![Screenshot](https://github.com/user-attachments/assets/706df4f9-0eea-4422-b319-a697f5eba0b6)

---

## 🛠️ Technologies Used

- **Rust**: For backend logic and game performance.
- **HTML**: For structuring the game interface.
- **CSS**: For styling and responsive design.

---

## ⚙️ How to Execute

To run **Bullet Math** locally, follow these steps:

### Prerequisites
- **Rust**: Install Rust from [https://www.rust-lang.org/](https://www.rust-lang.org/).
- **Python**: Install Python from [https://www.python.org/](https://www.python.org/).

### Backend (Rust)
1. Clone the repository:
   ```bash
   git clone https://github.com/danielyousif1/bullet-math.git
   cd bullet-math
   ```
2. Run the backend server:
   ```bash
   cargo run
   ```
   This will start the Rust backend server.

### Frontend (Python)
1. Run the Python HTTP server:
   ```bash
   python -m http.server 8000
   ```
   This will serve the frontend files on `http://localhost:8000`.

2. Open your browser and navigate to `http://localhost:8000` to play the game locally.

---

## 🚀 How to Deploy on Heroku

To deploy **Bullet Math** on Heroku, follow these steps:

### Prerequisites
- Install the [Heroku CLI](https://devcenter.heroku.com/articles/heroku-cli).

### Steps
1. Log in to your Heroku account:
   ```bash
   heroku login
   ```

2. Clone the repository (if you haven't already):
   ```bash
   heroku git:clone -a bullet-math
   cd bullet-math
   ```

3. Deploy your changes:
   ```bash
   git add .
   git commit -am "Deploying Bullet Math"
   git push heroku main
   ```

4. Open the app in your browser:
   ```bash
   heroku open
   ```

Your game will now be live on Heroku! 🎉

---

## 📂 Repository Structure

```
bullet-math/
├── src/            # Rust source code
├── public/         # HTML and CSS files
├── assets/         # Images and other static files
├── Cargo.toml      # Rust project configuration
└── README.md       # Project documentation
```

---

## 🤝 Contributing

Contributions are welcome! If you'd like to improve the game, feel free to open an issue or submit a pull request.

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/YourFeature`).
3. Commit your changes (`git commit -m 'Add some feature'`).
4. Push to the branch (`git push origin feature/YourFeature`).
5. Open a pull request.

---

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Inspired by [ZetaMac](https://arithmetic.zetamac.com/).
- Built with ❤️ using Rust, HTML, and CSS.

---

Ready to test your math skills? [Play Now!](https://bullet-math-4ed37ad30368.herokuapp.com/) 🎉

---

### Notes:
- If the frontend doesn’t require Python and is served directly by the Rust backend, you can remove the Python-related instructions.
- If the frontend setup is different (e.g., using a specific tool like `npm` or `yarn`), let me know, and I can adjust the instructions accordingly!

---

This version includes a clear and concise **How to Deploy on Heroku** section, making it easy for others to deploy your project. Let me know if you need further adjustments! 🚀
