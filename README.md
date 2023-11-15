# peer2package

A distributed package manager that encourages organisations to cache their package dependencies locally and share them with others.

## Design ideas

### 4 types of resource

1. Package - metadata about a package.
2. Package@Version - a specific instance of a package at a particular version
3. User - a public key that is used to sign resources
4. Certification - A user signature that encodes trust about a package or a user.

### Distributed Hash table

Use Kademlia with blake3 hash function.

### RPC

Using QUIC with self-signed certificates/mTLS. There's no use for unencrypted traffic anymore. This will hopefully make it less vulnerable to ISP discrimination if it looks like HTTP3/WebTransport traffic. If using DNS addresses, then server certificates must be signed by a CA.

### Authentication

Packages can be private but distributed globally with encryption. The certificates can be verified by
the service and the key can be returned
