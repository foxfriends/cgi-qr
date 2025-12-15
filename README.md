# CGI QR

Drop this thing into `cgi-bin` and have a QR generator up in minutes.

The path (with leading `/` dropped) is used as the QR data, with the query string used to provide various options. Refer to `main.rs` for that info.

An example configuration for Caddy, with [caddy-cgi](https://github.com/aksdb/caddy-cgi) installed:

```
{
    order cgi after file_server
}

example.com {
    file_server / {
        root /var/www/cgi-qr
        index index.html
    }
    cgi /* /usr/local/cgi-bin/cgi-qr
}
```
