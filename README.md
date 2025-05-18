# Install NixOS with `nixos-anywhere` and `sops`

## Requirements

- target must run on linux
- target must be accessible via ssh (passphrase or private key)
- target root access must be available

## Arguments to pass

- target hostname
- target destination
- target user
- target ssh key path
- target password (optional if ssh key is passed)
- target port

## Steps

1. setup and run nixos-anywhere
   1. remove old target ssh fingerprint
   2. generate new target ssh
   3. generate hardware config (optional)
   4. run nixos-anywhere
   5. reconnect ssh
   6. adding new target ssh fingerprint
   7. make some files persistent on target
2. generate age key for sops secrets
   1. generate target host (ssh-based) age key
   2. generate target user age key
   3. update sops file with new keys
3. copy config to target
4. build the full config

# DEBUG

`cargo run -- -n octopus -d localhost -u nixos -p me --port 10022 --config  /home/wallago/nix-config/`
