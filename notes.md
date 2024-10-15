[Server] - [Client]

# Descriptions
## Server
- Has web interface to allow users to enter the game
- Serves to the local area network (more options to come)
- Serves a qr code which will direct clients to the appropriate IP and port
- Generates security code that needs to be entered by client

## Client
- Connect to server through browser after scanning qr code
- Verify security code from server through prompt

## QR Code
- Contains IP address and port number of the server

# Duties of Server
- Create qr code
- Serve html pages to clients
- Embed game state in client
- Accept new clients
