Ketika mengetik teks di satu klien, pesan dikirim ke server, yang kemudian menyiarkannya ke semua klien lain yang terhubung. Server menggunakan channel `tokio::sync::broadcast`, dan klien/server menggunakan `tokio::select!` untuk operasi bersamaan.

![alt text](<Screenshot 2025-05-23 181317.png>)

Saya memodifikasi port WebSocket dari `2000` menjadi `8080`.
- Di `src/bin/server.rs`, saya mengubah string alamat dalam `TcpListener::bind()`.
- Di `src/bin/client.rs`, saya mengubah variabel `server_addr`.