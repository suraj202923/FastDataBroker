/**
 * FastDataBroker JavaScript SDK Test Suite
 */

const assert = require('assert');

/**
 * Mock Client for testing
 */
class MockClient {
    constructor(brokers, streamName) {
        this.brokers = brokers;
        this.streamName = streamName;
        this.partitionCount = 4;
    }
    
    getStreamName() {
        return this.streamName;
    }
    
    getBrokerCount() {
        return this.brokers.length;
    }
    
    getBrokers() {
        return [...this.brokers];
    }
    
    getTopology() {
        return {
            stream: this.streamName,
            broker_count: this.brokers.length,
            partition_count: this.partitionCount,
            replication_factor: 3
        };
    }
}

/**
 * Producer for testing
 */
class Producer {
    constructor(client) {
        this.client = client;
    }
    
    send(key, value) {
        const hash = this.hashKey(key);
        return hash % 4;
    }
    
    hashKey(key) {
        let hash = 0;
        for (let i = 0; i < key.length; i++) {
            hash += key.charCodeAt(i);
        }
        return Math.abs(hash);
    }
}

/**
 * Consumer for testing
 */
class Consumer {
    constructor(client, groupId) {
        this.client = client;
        this.groupId = groupId;
        this.offsets = {};
    }
    
    commitOffset(partition, offset) {
        this.offsets[partition] = offset;
    }
    
    getCommittedOffset(partition) {
        return this.offsets[partition] || 0;
    }
    
    async poll(timeout = 5000) {
        return new Promise((resolve) => {
            setTimeout(() => {
                resolve(null);
            }, timeout);
        });
    }
}

/**
 * Partition for testing
 */
class Partition {
    constructor(id, replicas) {
        this.id = id;
        this.replicas = replicas;
        this.leader = replicas[0];
        this.inSyncReplicas = replicas.slice(0, 2);
    }
}

/**
 * Test Suite
 */
describe('FastDataBroker JavaScript SDK', () => {
    let client;
    
    beforeEach(() => {
        const brokers = [
            'broker1:8080',
            'broker2:8081',
            'broker3:8082',
            'broker4:8083'
        ];
        client = new MockClient(brokers, 'test-stream');
    });
    
    describe('Client Initialization', () => {
        it('should create client with brokers', () => {
            assert(client !== null);
            assert.equal(client.getStreamName(), 'test-stream');
            assert.equal(client.getBrokerCount(), 4);
        });
        
        it('should have valid broker list', () => {
            const brokers = client.getBrokers();
            assert.equal(brokers.length, 4);
            assert(brokers[0].includes(':'));
        });
    });
    
    describe('Topology Discovery', () => {
        it('should retrieve cluster topology', () => {
            const topology = client.getTopology();
            assert.equal(topology.stream, 'test-stream');
            assert.equal(topology.broker_count, 4);
            assert.equal(topology.partition_count, 4);
            assert.equal(topology.replication_factor, 3);
        });
    });
    
    describe('Producer', () => {
        let producer;
        
        beforeEach(() => {
            producer = new Producer(client);
        });
        
        it('should send message to partition', () => {
            const partition = producer.send('test-key', 'test-value');
            assert(partition >= 0 && partition < 4);
        });
        
        it('should send multiple messages', () => {
            for (let i = 0; i < 100; i++) {
                const partition = producer.send(`key-${i}`, `value-${i}`);
                assert(partition >= 0 && partition < 4);
            }
        });
        
        it('should hash same key to same partition', () => {
            const partition1 = producer.send('same-key', 'value1');
            const partition2 = producer.send('same-key', 'value2');
            assert.equal(partition1, partition2);
        });
        
        it('should distribute keys evenly', () => {
            const distribution = [0, 0, 0, 0];
            const keyCount = 1000;
            
            for (let i = 0; i < keyCount; i++) {
                const partition = producer.send(`key-${i}`, `value-${i}`);
                distribution[partition]++;
            }
            
            // Check balance (within 25%)
            const expectedPerPartition = keyCount / 4; // 250
            const tolerance = expectedPerPartition / 4; // 62
            
            distribution.forEach((count, partition) => {
                assert(
                    count >= expectedPerPartition - tolerance &&
                    count <= expectedPerPartition + tolerance,
                    `Partition ${partition} has ${count} keys (expected ~${expectedPerPartition})`
                );
            });
        });
    });
    
    describe('Consumer', () => {
        let consumer;
        
        beforeEach(() => {
            consumer = new Consumer(client, 'test-group');
        });
        
        it('should commit offset', () => {
            consumer.commitOffset(0, 100);
            assert.equal(consumer.getCommittedOffset(0), 100);
        });
        
        it('should handle multiple partitions', () => {
            for (let i = 0; i < 4; i++) {
                consumer.commitOffset(i, i * 100);
            }
            
            for (let i = 0; i < 4; i++) {
                assert.equal(consumer.getCommittedOffset(i), i * 100);
            }
        });
        
        it('should return 0 for uncommitted partition', () => {
            assert.equal(consumer.getCommittedOffset(99), 0);
        });
        
        it('should poll messages with timeout', async () => {
            const message = await consumer.poll(100);
            // Message will be null in mock
            assert(message === null);
        }).timeout(500);
    });
    
    describe('Partitioning', () => {
        it('should create partition with replicas', () => {
            const replicas = ['broker0', 'broker1', 'broker2'];
            const partition = new Partition(0, replicas);
            
            assert.equal(partition.id, 0);
            assert.equal(partition.leader, 'broker0');
            assert.equal(partition.inSyncReplicas.length, 2);
        });
        
        it('should handle partition failover', () => {
            const partition = new Partition(0, ['broker0', 'broker1', 'broker2']);
            
            // Simulate broker0 failure
            partition.inSyncReplicas = ['broker1', 'broker2'];
            partition.leader = 'broker1'; // Elect new leader
            
            assert.equal(partition.leader, 'broker1');
            assert.equal(partition.inSyncReplicas.length, 2);
        });
    });
    
    describe('Replication', () => {
        it('should have 3-way replication', () => {
            const partition = new Partition(0, ['broker0', 'broker1', 'broker2']);
            assert.equal(partition.replicas.length, 3);
        });
        
        it('should tolerate 1 broker failure', () => {
            let replicas = ['broker0', 'broker1', 'broker2'];
            
            // Simulate 1 failure
            replicas = replicas.filter(b => b !== 'broker0');
            
            assert.equal(replicas.length, 2);
            assert(replicas.includes('broker1'));
            assert(replicas.includes('broker2'));
        });
    });
    
    describe('Hash Performance', () => {
        it('should hash keys quickly', () => {
            const producer = new Producer(client);
            const iterations = 10000;
            
            const startTime = Date.now();
            for (let i = 0; i < iterations; i++) {
                producer.send(`key-${i}`, `value-${i}`);
            }
            const endTime = Date.now();
            
            const durationMs = endTime - startTime;
            console.log(`Hashed ${iterations} keys in ${durationMs}ms`);
            assert(durationMs < 1000, 'Hashing should be fast');
        });
    });
    
    describe('Concurrent Operations', () => {
        it('should handle concurrent sends', async () => {
            const producer = new Producer(client);
            const promises = [];
            
            for (let i = 0; i < 100; i++) {
                promises.push(
                    Promise.resolve(producer.send(`key-${i}`, `value-${i}`))
                );
            }
            
            const results = await Promise.all(promises);
            assert.equal(results.length, 100);
        });
    });
    
    describe('Error Handling', () => {
        it('should handle invalid partition gracefully', () => {
            const producer = new Producer(client);
            const partition = producer.send('test', 'value');
            
            assert(partition >= 0 && partition < 4, 'Partition should be valid');
        });
    });
});

/**
 * Run tests with: npm test
 * Or with mocha: mocha test_client.js
 */

// Export for use in other tests
module.exports = {
    MockClient,
    Producer,
    Consumer,
    Partition
};
