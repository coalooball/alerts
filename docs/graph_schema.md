# 告警关联图谱数据模型

## 节点类型 (Nodes)

### 1. Alert (告警节点)
```cypher
(:Alert {
  id: String,           // 告警ID
  type: String,         // EDR/NGAV/DNS等
  severity: Integer,    // 严重程度
  create_time: DateTime,
  status: String,
  org_key: String
})
```

### 2. Device (设备节点)
```cypher
(:Device {
  device_id: String,
  device_name: String,
  device_os: String,
  internal_ip: String,
  external_ip: String,
  org_key: String
})
```

### 3. Process (进程节点)
```cypher
(:Process {
  process_guid: String,
  process_path: String,
  process_name: String,
  process_hash: String,
  process_cmdline: String,
  process_pid: Integer,
  username: String,
  reputation: String
})
```

### 4. IOC (威胁指标节点)
```cypher
(:IOC {
  ioc_id: String,
  ioc_type: String,    // IP/Domain/Hash/URL
  ioc_value: String,
  threat_score: Integer,
  tags: [String]
})
```

### 5. ThreatActor (威胁组织节点)
```cypher
(:ThreatActor {
  actor_id: String,
  actor_name: String,
  apt_group: String,
  country: String,
  tactics: [String]
})
```

### 6. MitreAttack (MITRE ATT&CK节点)
```cypher
(:MitreAttack {
  technique_id: String,  // T1055
  technique_name: String,
  tactic: String,
  description: String
})
```

### 7. User (用户节点)
```cypher
(:User {
  username: String,
  user_sid: String,
  department: String,
  privilege_level: String
})
```

## 关系类型 (Relationships)

### 1. TRIGGERED_ON (告警触发于设备)
```cypher
(alert:Alert)-[:TRIGGERED_ON {timestamp: DateTime}]->(device:Device)
```

### 2. INVOLVES_PROCESS (告警涉及进程)
```cypher
(alert:Alert)-[:INVOLVES_PROCESS {role: "target|source"}]->(process:Process)
```

### 3. PARENT_OF (父子进程关系)
```cypher
(parent:Process)-[:PARENT_OF {spawn_time: DateTime}]->(child:Process)
```

### 4. COMMUNICATES_WITH (进程通信关系)
```cypher
(process1:Process)-[:COMMUNICATES_WITH {
  protocol: String,
  port: Integer,
  bytes_sent: Integer
}]->(process2:Process)
```

### 5. CONTAINS_IOC (告警包含IOC)
```cypher
(alert:Alert)-[:CONTAINS_IOC {confidence: Float}]->(ioc:IOC)
```

### 6. EXECUTED_BY (进程由用户执行)
```cypher
(process:Process)-[:EXECUTED_BY]->(user:User)
```

### 7. USES_TECHNIQUE (使用ATT&CK技术)
```cypher
(alert:Alert)-[:USES_TECHNIQUE]->(mitre:MitreAttack)
```

### 8. ATTRIBUTED_TO (归因于威胁组织)
```cypher
(alert:Alert)-[:ATTRIBUTED_TO {confidence: Float}]->(actor:ThreatActor)
```

### 9. RELATED_TO (告警关联关系)
```cypher
(alert1:Alert)-[:RELATED_TO {
  correlation_type: String,  // temporal|spatial|behavioral
  correlation_score: Float,
  reason: String
}]->(alert2:Alert)
```

## 常用查询示例

### 1. 查找某设备上的所有关联告警
```cypher
MATCH (d:Device {device_id: $deviceId})<-[:TRIGGERED_ON]-(a:Alert)
RETURN a ORDER BY a.create_time DESC
```

### 2. 查找告警的完整攻击链
```cypher
MATCH path = (a:Alert {id: $alertId})-[:INVOLVES_PROCESS]->
  (p:Process)-[:PARENT_OF*0..5]->(child:Process)
RETURN path
```

### 3. 查找使用相同IOC的所有告警
```cypher
MATCH (ioc:IOC {ioc_value: $iocValue})<-[:CONTAINS_IOC]-(a:Alert)
RETURN a, ioc
```

### 4. 查找横向移动路径
```cypher
MATCH path = (d1:Device)-[:TRIGGERED_ON]-(a1:Alert)-[:RELATED_TO*1..3]
  -(a2:Alert)-[:TRIGGERED_ON]-(d2:Device)
WHERE d1 <> d2
RETURN path
```

### 5. 查找kill chain完整路径
```cypher
MATCH (a:Alert)-[:USES_TECHNIQUE]->(m:MitreAttack)
WITH a, m ORDER BY a.create_time
MATCH path = (a)-[:RELATED_TO*0..10]-(related:Alert)
RETURN path, collect(DISTINCT m.tactic) as tactics
```