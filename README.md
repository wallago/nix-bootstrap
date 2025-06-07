# 🚀 Nix Remote Bootstrapper

A CLI tool for **bootstrapping remote [NixOS](https://nixos.org/) hosts over SSH** using [`nixos-anywhere`](https://github.com/nix-community/nixos-anywhere), with support for:

- 🛠️ Generating a `hardware-configuration.nix` from a remote system
- 📦 Running `nixos-anywhere` to deploy a NixOS flake
- 🔑 Updating sops and SSH keys for `age` encryption and SSH access

## ✨ Features

- 🧑‍💻 Interactive prompts for SSH target and port
- 🧬 Hardware configuration generation via `nixos-generate-config`
- 📡 Flake-based deployment using `nixos-anywhere`
- 🔐 Automatic `age` key generation from the host's SSH key
- 📝 Update `.sops.yaml` and `ssh_host_ed25519_key.pub`
-

## 📦 Requirements

- 🦀 [Rust](https://www.rust-lang.org/tools/install)
- ❄️ `[nixos-anywhere](https://github.com/nix-community/nixos-anywhere)`
- 🔐 `[sops](https://github.com/getsops/sops)`
- 🧬 A flake-based NixOS configuration (hosts/{hostname} structure)
- 🔑 SSH access to the target machine

## 🚀 Usage

```bash
cargo run -- \
  --config-hostname myhost \
  --config-path /absolute/path/to/nix/flake
```

---

🧱 Preparation Phase

1. ✅ Enter target destination and port
2. ✅ Enter target username
3. ✅ Establish SSH connection

🏗️ Build Nix Starter Config for Target

4. ❌ Generate temporary local directory
5. ❌ Git clone nix-starter-config into it
6. ❌ Generate hardware-configuration.nix on target and save it into flake
7. ❌ Add target’s SSH public key to the flake

🚀 Initial Deployment with Minimal Config

8. ❌ Deploy nix-starter-config into target (with nixos-anywhere)

🔐 Secrets Setup Phase

9. ✅ Reconnect to new target system
10. ✅ Generate age key from target SSH ed25519 key
11. ✅ Update .sops.yaml and ssh_host_ed25519_key.pub in flake

🧩 Final Deployment

12. ❌ Deploy real full config (with secrets) into target

---

1. ✅ enter target destination and port
2. ✅ enter target username
3. ✅ establishe ssh connection
4. ❌ generate host tmp dir
5. ❌ git clone nix-starter-config into it
6. ❌ update nix-starter-config `hardware-configuration.nix` with `nixos-generate-config` from target
7. ❌ update nix-starter-config to know host ssh pub key
8. ❌ deploy nix starter config with `nixos-anywhere` into target
9. ✅ reconnect ssh connection with the new config
10. ✅ get ssh ed25519 key and generate `age` key
11. ✅ update `.sops.yaml` and `ssh_host_ed25519_key.pub`
12. ❌ use `nixos-anywhere` or something else to deploy the final nix config into target

---

- target must run on linux
- target must be accessible via ssh (passphrase or private key)
- target root access must be available

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

---

## Steps

### From nix ISO

- connect to remote in ssh
- clone locally nix starter config
- generate hardware configuration
- select a disk device
- replace value of the nix starter config with those infos
- deploy with nixos-anywhere the config

### From nix whatever config

- connect to remote in ssh
- clone locally nix config
- generate an age key from the remote ssh key
- add it to .sops.yaml
- update secrets file to add the new age key to decrypt it
- replace value of the nix config with those infos
- deploy FAILING ...
