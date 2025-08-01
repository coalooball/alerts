#!/usr/bin/env python3
"""Generate test EDR and NGAV alert data based on sample files."""

import json
import random
import uuid
from datetime import datetime, timedelta
import argparse
from pathlib import Path

def generate_edr_alerts(num_records=10000):
    """Generate EDR alerts based on sample structure."""
    
    # Sample data variations
    device_names = [f"WIN-{i}-H{j}" for i in range(10, 50) for j in range(1, 5)]
    device_ips_internal = [f"192.168.{i}.{j}" for i in range(1, 10) for j in range(10, 250)]
    device_ips_external = [f"130.126.{i}.{j}" for i in range(200, 256) for j in range(1, 250)]
    
    process_names = ["cmd.exe", "powershell.exe", "python.exe", "chrome.exe", "firefox.exe", 
                     "excel.exe", "winword.exe", "notepad.exe", "svchost.exe", "explorer.exe"]
    
    report_names = [
        "Execution - Command and Scripting Interpreter Execution",
        "Discovery - packet capture tools",
        "Execution - SysInternals Use",
        "Persistence - Regmod Run or Runonce Key Modification",
        "Defense Evasion - Process Injection",
        "Credential Access - Credential Dumping",
        "Lateral Movement - Remote Services",
        "Collection - Data from Local System",
        "Exfiltration - Data Transfer Size Limits",
        "Impact - Service Stop"
    ]
    
    watchlist_names = ["ATT&CK Framework", "Carbon Black Endpoint Visibility", 
                       "Carbon Black Endpoint Suspicious Indicators", "Custom Threat Intelligence"]
    
    severities = [1, 2, 3, 4]
    severity_weights = [0.1, 0.2, 0.4, 0.3]  # 1=10%, 2=20%, 3=40%, 4=30%
    
    alerts = []
    base_time = datetime.utcnow() - timedelta(days=7)
    
    for i in range(num_records):
        # Generate timestamps with some temporal clustering
        time_offset = timedelta(
            seconds=random.randint(0, 7 * 24 * 3600),
            microseconds=random.randint(0, 999999)
        )
        create_time = base_time + time_offset
        
        device_id = random.randint(90000000, 99999999)
        device_name = random.choice(device_names)
        
        alert = {
            "schema": 1,
            "create_time": create_time.strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z",
            "device_external_ip": random.choice(device_ips_external),
            "device_id": device_id,
            "device_internal_ip": random.choice(device_ips_internal),
            "device_name": device_name,
            "device_os": "WINDOWS",
            "ioc_hit": f"(process_name:{random.choice(process_names)}* OR process_cmdline:*suspicious*) -enriched:true",
            "ioc_id": f"{uuid.uuid4()}-0",
            "org_key": "7DMF69PK",
            "parent_cmdline": f"C:\\Windows\\system32\\{random.choice(['services.exe', 'explorer.exe', 'svchost.exe'])}",
            "parent_guid": f"7DMF69PK-{uuid.uuid4().hex[:8]}-{random.randint(1000, 9999):08x}-00000000-{uuid.uuid4().hex[:15]}",
            "parent_hash": [uuid.uuid4().hex, uuid.uuid4().hex[:64]],
            "parent_path": f"c:\\windows\\system32\\{random.choice(['services.exe', 'explorer.exe', 'svchost.exe'])}",
            "parent_pid": random.randint(100, 9999),
            "parent_publisher": [{
                "name": "Microsoft Windows",
                "state": "FILE_SIGNATURE_STATE_SIGNED | FILE_SIGNATURE_STATE_VERIFIED | FILE_SIGNATURE_STATE_TRUSTED | FILE_SIGNATURE_STATE_OS"
            }],
            "parent_reputation": "REP_WHITE",
            "parent_username": f"{device_name}\\{random.choice(['admin', 'user', 'aalsahee', 'test', 'developer'])}",
            "process_cmdline": f"C:\\{random.choice(['Windows\\system32', 'Program Files', 'Users\\Public'])}\\{random.choice(process_names)} {random.choice(['/c', '-ExecutionPolicy Bypass', '--version', '/k', ''])}",
            "process_guid": f"7DMF69PK-{uuid.uuid4().hex[:8]}-{random.randint(1000, 9999):08x}-00000000-{uuid.uuid4().hex[:15]}",
            "process_hash": [uuid.uuid4().hex, uuid.uuid4().hex[:64]],
            "process_path": f"c:\\{random.choice(['windows\\system32', 'program files', 'users\\public'])}\\{random.choice(process_names)}",
            "process_pid": random.randint(1000, 9999),
            "process_publisher": [{
                "name": random.choice(["Microsoft Corporation", "Mozilla Corporation", "Google LLC", "Unknown"]),
                "state": "FILE_SIGNATURE_STATE_SIGNED | FILE_SIGNATURE_STATE_VERIFIED | FILE_SIGNATURE_STATE_TRUSTED"
            }],
            "process_reputation": random.choice(["REP_WHITE", "REP_WHITE", "REP_WHITE", "REP_NEUTRAL"]),
            "process_username": f"{device_name}\\{random.choice(['admin', 'user', 'aalsahee', 'test', 'developer'])}",
            "report_id": f"{uuid.uuid4().hex[:22]}-{uuid.uuid4()}",
            "report_name": random.choice(report_names),
            "report_tags": ["attack", "attackframework", "threathunting", "windows", 
                           f"t{random.randint(1000, 1600)}", random.choice(["execution", "persistence", "defense-evasion", "discovery"])],
            "severity": random.choices(severities, weights=severity_weights)[0],
            "type": "watchlist.hit",
            "watchlists": [{
                "id": uuid.uuid4().hex[:22],
                "name": random.choice(watchlist_names)
            }]
        }
        
        alerts.append(alert)
    
    return alerts

def generate_ngav_alerts(num_records=10000):
    """Generate NGAV alerts based on sample structure."""
    
    # Sample data variations
    device_names = [f"WIN-{i}-H{j}" for i in range(10, 50) for j in range(1, 5)]
    device_ips_internal = [f"192.168.{i}.{j}" for i in range(1, 10) for j in range(10, 250)]
    device_ips_external = [f"130.126.{i}.{j}" for i in range(200, 256) for j in range(1, 250)]
    
    process_names = ["python.exe", "powershell.exe", "cmd.exe", "chrome.exe", "firefox.exe", 
                     "excel.exe", "winword.exe", "java.exe", "node.exe", "ruby.exe"]
    
    reasons = [
        "The application {} acted as a network server.",
        "The application {} attempted to inject into another process.",
        "The application {} accessed credentials.",
        "The application {} modified system settings.",
        "The application {} created suspicious files.",
        "The application {} established external connections.",
        "The application {} exhibited ransomware-like behavior.",
        "The application {} attempted privilege escalation."
    ]
    
    reason_codes = ["R_NET_SERVER", "R_PROCESS_INJECT", "R_CRED_ACCESS", "R_SYS_MOD", 
                    "R_FILE_CREATE", "R_NET_CONN", "R_RANSOMWARE", "R_PRIV_ESC"]
    
    threat_categories = ["NON_MALWARE", "POTENTIALLY_UNWANTED", "SUSPICIOUS", "MALWARE"]
    severities = [1, 2, 3, 4, 5]
    severity_weights = [0.05, 0.15, 0.4, 0.3, 0.1]  # 1=5%, 2=15%, 3=40%, 4=30%, 5=10%
    
    ttps = ["FIXED_PORT_LISTEN", "ACTIVE_SERVER", "NETWORK_ACCESS", "PROCESS_INJECTION",
            "CREDENTIAL_THEFT", "PERSISTENCE", "DEFENSE_EVASION", "LATERAL_MOVEMENT"]
    
    alerts = []
    base_time = datetime.utcnow() - timedelta(days=7)
    
    for i in range(num_records):
        # Generate timestamps
        time_offset = timedelta(
            seconds=random.randint(0, 7 * 24 * 3600),
            microseconds=random.randint(0, 999999)
        )
        create_time = base_time + time_offset
        last_update_time = create_time + timedelta(seconds=random.randint(10, 300))
        first_event_time = create_time - timedelta(seconds=random.randint(0, 60))
        last_event_time = create_time
        
        device_id = random.randint(90000000, 99999999)
        device_name = random.choice(device_names)
        process_name = random.choice(process_names)
        
        severity = random.choices(severities, weights=severity_weights)[0]
        threat_category = random.choice(threat_categories)
        
        alert = {
            "type": "CB_ANALYTICS",
            "id": str(uuid.uuid4()),
            "legacy_alert_id": str(uuid.uuid4()),
            "org_key": "7DMF69PK",
            "create_time": create_time.strftime("%Y-%m-%dT%H:%M:%SZ"),
            "last_update_time": last_update_time.strftime("%Y-%m-%dT%H:%M:%SZ"),
            "first_event_time": first_event_time.strftime("%Y-%m-%dT%H:%M:%SZ"),
            "last_event_time": last_event_time.strftime("%Y-%m-%dT%H:%M:%SZ"),
            "threat_id": uuid.uuid4().hex,
            "severity": severity,
            "category": random.choice(["THREAT", "NOTICE", "WARNING"]),
            "device_id": device_id,
            "device_os": "WINDOWS",
            "device_os_version": f"Windows {random.choice(['7', '10', '11'])} x{random.choice(['86', '64'])} SP: 1",
            "device_name": device_name,
            "device_username": f"{random.choice(['admin', 'user', 'test', 'developer'])}@example.com",
            "policy_id": random.randint(260000, 270000),
            "policy_name": random.choice(["Standard", "High Security", "Development", "Production"]),
            "target_value": random.choice(["LOW", "MEDIUM", "HIGH"]),
            "workflow": {
                "state": "OPEN",
                "remediation": "",
                "last_update_time": last_update_time.strftime("%Y-%m-%dT%H:%M:%SZ"),
                "comment": "",
                "changed_by": "Carbon Black"
            },
            "device_internal_ip": random.choice(device_ips_internal),
            "device_external_ip": random.choice(device_ips_external),
            "alert_url": f"https://defense-prod05.conferdeploy.net/triage?incidentId={uuid.uuid4()}&orgId=35152",
            "reason": random.choice(reasons).format(process_name),
            "reason_code": random.choice(reason_codes),
            "process_name": process_name,
            "device_location": random.choice(["ONSITE", "OFFSITE", "UNKNOWN"]),
            "created_by_event_id": uuid.uuid4().hex,
            "threat_indicators": [
                {
                    "process_name": process_name,
                    "sha256": uuid.uuid4().hex + uuid.uuid4().hex[:32],
                    "ttps": random.sample(ttps, k=random.randint(1, 3))
                }
            ],
            "threat_cause_actor_sha256": uuid.uuid4().hex + uuid.uuid4().hex[:32],
            "threat_cause_actor_name": process_name,
            "threat_cause_actor_process_pid": f"{random.randint(1000, 9999)}-{random.randint(100000000000000000, 999999999999999999)}-0",
            "threat_cause_reputation": random.choice(["TRUSTED_WHITE_LIST", "KNOWN_GOOD", "NEUTRAL", "SUSPECT"]),
            "threat_cause_threat_category": threat_category,
            "threat_cause_vector": random.choice(["UNKNOWN", "EMAIL", "WEB", "USB", "NETWORK"]),
            "threat_cause_cause_event_id": uuid.uuid4().hex,
            "blocked_threat_category": "UNKNOWN" if severity < 3 else threat_category,
            "not_blocked_threat_category": threat_category if severity < 3 else "UNKNOWN",
            "kill_chain_status": random.sample(["INSTALL_RUN", "EXECUTE", "DISCOVER", "EXPLOIT"], k=random.randint(1, 2)),
            "run_state": "RAN" if severity < 3 else random.choice(["RAN", "BLOCKED"]),
            "policy_applied": "NOT_APPLIED" if severity < 3 else random.choice(["NOT_APPLIED", "APPLIED"])
        }
        
        alerts.append(alert)
    
    return alerts

def main():
    parser = argparse.ArgumentParser(description='Generate test EDR and NGAV alert data')
    parser.add_argument('--edr-count', type=int, default=10000, help='Number of EDR alerts to generate')
    parser.add_argument('--ngav-count', type=int, default=10000, help='Number of NGAV alerts to generate')
    parser.add_argument('--output-dir', type=str, default='fixtures', help='Output directory for generated files')
    
    args = parser.parse_args()
    
    output_dir = Path(args.output_dir)
    output_dir.mkdir(exist_ok=True)
    
    # Generate EDR alerts
    print(f"Generating {args.edr_count} EDR alerts...")
    edr_alerts = generate_edr_alerts(args.edr_count)
    edr_file = output_dir / "edr-alerts-test-10k.jsonl"
    
    with open(edr_file, 'w') as f:
        for alert in edr_alerts:
            f.write(json.dumps(alert) + '\n')
    
    print(f"✅ Generated {len(edr_alerts)} EDR alerts in {edr_file}")
    
    # Generate NGAV alerts
    print(f"Generating {args.ngav_count} NGAV alerts...")
    ngav_alerts = generate_ngav_alerts(args.ngav_count)
    ngav_file = output_dir / "ngav-alerts-test-10k.jsonl"
    
    with open(ngav_file, 'w') as f:
        for alert in ngav_alerts:
            f.write(json.dumps(alert) + '\n')
    
    print(f"✅ Generated {len(ngav_alerts)} NGAV alerts in {ngav_file}")

if __name__ == "__main__":
    main()