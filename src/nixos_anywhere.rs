use anyhow::Result;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
};

use crate::Params;

pub fn run(params: &Params) -> Result<()> {
    remove_known_hosts_entries(&params)?;

    println!(
        "Installing NixOS on remote host $target_hostname at {}",
        params.target_destination
    );
    println!("Preparing a new ssh_host_ed25519_key pair for $target_hostname");
    Ok(())
}

fn remove_known_hosts_entries(params: &Params) -> Result<()> {
    println!("Wiping known_hosts of {}", params.target_destination);
    let patterns = [&params.target_hostname, &params.target_destination].to_vec();
    let file_in = fs::File::open("~/.ssh/known_hosts")?;
    let reader = BufReader::new(file_in);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !patterns.iter().any(|pat| line.contains(*pat)))
        .collect();

    // Overwrite the file with filtered lines
    let mut file_out = fs::File::create("~/.ssh/known_hosts")?;
    for line in lines {
        writeln!(file_out, "{line}")?;
    }
    Ok(())
}

// 	green "Installing NixOS on remote host $target_hostname at $target_destination"

// 	###
// 	# nixos-anywhere extra-files generation
// 	###
// 	green "Preparing a new ssh_host_ed25519_key pair for $target_hostname."
// 	# Create the directory where sshd expects to find the host keys
// 	install -d -m755 "$temp/$persist_dir/etc/ssh"

// 	# Generate host ssh key pair without a passphrase
// 	ssh-keygen -t ed25519 -f "$temp/$persist_dir/etc/ssh/ssh_host_ed25519_key" -C "$target_user"@"$target_hostname" -N ""

// 	# Set the correct permissions so sshd will accept the key
// 	chmod 600 "$temp/$persist_dir/etc/ssh/ssh_host_ed25519_key"

// 	green "Adding ssh host fingerprint at $target_destination to ~/.ssh/known_hosts"
// 	# This will fail if we already know the host, but that's fine
// 	ssh-keyscan -p "$ssh_port" "$target_destination" | grep -v '^#' >>~/.ssh/known_hosts || true

// 	###
// 	# nixos-anywhere installation
// 	###
// 	cd nixos-installer
// 	# when using luks, disko expects a passphrase on /tmp/disko-password, so we set it for now and will update the passphrase later
// 	if no_or_yes "Manually set luks encryption passphrase? (Default: \"$luks_passphrase\")"; then
// 		blue "Enter your luks encryption passphrase:"
// 		read -rs luks_passphrase
// 		$ssh_root_cmd "/bin/sh -c 'echo $luks_passphrase > /tmp/disko-password'"
// 	else
// 		green "Using '$luks_passphrase' as the luks encryption passphrase. Change after installation."
// 		$ssh_root_cmd "/bin/sh -c 'echo $luks_passphrase > /tmp/disko-password'"
// 	fi
// 	# this will run if luks_secondary_drive_labels cli argument was set, regardless of whether the luks_passphrase is default or not
// 	if [ -n "${luks_secondary_drive_labels}" ]; then
// 		luks_setup_secondary_drive_decryption
// 	fi

// 	# If you are rebuilding a machine without any hardware changes, this is likely unneeded or even possibly disruptive
// 	if no_or_yes "Generate a new hardware config for this host? Yes if your nix-config doesn't have an entry for this host."; then
// 		green "Generating hardware-configuration.nix on $target_hostname and adding it to the local nix-config."
// 		$ssh_root_cmd "nixos-generate-config --no-filesystems --root /mnt"
// 		$scp_cmd root@"$target_destination":/mnt/etc/nixos/hardware-configuration.nix \
// 			"${git_root}"/hosts/nixos/"$target_hostname"/hardware-configuration.nix
// 		generated_hardware_config=1
// 	fi

// 	# --extra-files here picks up the ssh host key we generated earlier and puts it onto the target machine
// 	SHELL=/bin/sh nix run github:nix-community/nixos-anywhere -- \
// 		--ssh-port "$ssh_port" \
// 		--post-kexec-ssh-port "$ssh_port" \
// 		--extra-files "$temp" \
// 		--flake .#"$target_hostname" \
// 		root@"$target_destination"

// 	if ! yes_or_no "Has your system restarted and are you ready to continue? (no exits)"; then
// 		exit 0
// 	fi

// 	green "Adding $target_destination's ssh host fingerprint to ~/.ssh/known_hosts"
// 	ssh-keyscan -p "$ssh_port" "$target_destination" | grep -v '^#' >>~/.ssh/known_hosts || true

// 	if [ -n "$persist_dir" ]; then
// 		$ssh_root_cmd "cp /etc/machine-id $persist_dir/etc/machine-id || true"
// 		$ssh_root_cmd "cp -R /etc/ssh/ $persist_dir/etc/ssh/ || true"
// 	fi
// 	cd - >/dev/null
// }
