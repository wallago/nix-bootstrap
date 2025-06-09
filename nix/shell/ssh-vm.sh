user="$1"

if [[ -z "$user" ]]; then
  echo "Usage: ssh-vm <user>"
  return 1
fi

ssh -p 10022 $user@localhost
