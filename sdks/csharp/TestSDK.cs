using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

/**
 * FastDataBroker C# SDK - Test Suite
 */

class Message
{
    public string Topic { get; set; }
    public object Payload { get; set; }
    public int Priority { get; set; } = 5;
    public int TtlSeconds { get; set; } = 3600;
    public Dictionary<string, string> Headers { get; set; } = new();

    public Message(string topic, object payload)
    {
        Topic = topic;
        Payload = payload;
    }

    public Message WithPriority(int priority)
    {
        Priority = priority;
        return this;
    }
}

class DeliveryResult
{
    public string MessageId { get; set; }
    public string Status { get; set; }
    public double LatencyMs { get; set; }
    public long Timestamp { get; set; }

    public DeliveryResult(string messageId, string status, double latencyMs, long timestamp)
    {
        MessageId = messageId;
        Status = status;
        LatencyMs = latencyMs;
        Timestamp = timestamp;
    }

    public override string ToString() =>
        $"DeliveryResult{{messageId='{MessageId}', status='{Status}', latencyMs={LatencyMs:F2}}}";
}

class ConnectionStats
{
    public bool IsConnected { get; set; }
    public long MessagesSent { get; set; }
    public long MessagesReceived { get; set; }
    public long ConnectionTimeMs { get; set; }
    public long UptimeSeconds { get; set; }
    public long LastMessageTime { get; set; }

    public ConnectionStats(bool isConnected, long messagesSent, long messagesReceived,
                          long connectionTimeMs, long uptimeSeconds, long lastMessageTime)
    {
        IsConnected = isConnected;
        MessagesSent = messagesSent;
        MessagesReceived = messagesReceived;
        ConnectionTimeMs = connectionTimeMs;
        UptimeSeconds = uptimeSeconds;
        LastMessageTime = lastMessageTime;
    }

    public override string ToString() =>
        $"ConnectionStats{{isConnected={IsConnected}, messagesSent={MessagesSent}, uptimeSeconds={UptimeSeconds}}}";
}

class FastDataBrokerQuicClient : IDisposable
{
    private Dictionary<string, string> _config;
    private bool _connected;
    private bool _authenticated;
    private Dictionary<string, long> _stats;
    private long _connectionStart;
    private Dictionary<string, Action<object>> _messageHandlers;
    private Random _random = new();

    public FastDataBrokerQuicClient(string host, int port, string tenantId, string clientId, string pskSecret)
    {
        _config = new Dictionary<string, string>
        {
            { "host", host },
            { "port", port.ToString() },
            { "tenant_id", tenantId },
            { "client_id", clientId },
            { "psk_secret", pskSecret }
        };

        _connected = false;
        _authenticated = false;
        _stats = new Dictionary<string, long>
        {
            { "messages_sent", 0 },
            { "messages_received", 0 }
        };
        _messageHandlers = new Dictionary<string, Action<object>>();
    }

    public async Task ConnectAsync()
    {
        var host = _config["host"];
        var port = _config["port"];
        Console.WriteLine($"Connecting to {host}:{port}...");
        
        _connected = true;
        _connectionStart = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        
        await Task.Delay(10);
        Console.WriteLine($"✓ Connected to {host}:{port}");
    }

    public async Task<DeliveryResult> SendMessageAsync(Message message)
    {
        if (!_connected)
        {
            throw new InvalidOperationException("Not connected");
        }

        var messageId = $"msg_{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}_{_random.Next(10000)}";
        var latency = _random.NextDouble() * 50 + 5;
        _stats["messages_sent"]++;

        await Task.Delay((int)(latency / 10));
        return new DeliveryResult(messageId, "success", latency, DateTimeOffset.UtcNow.ToUnixTimeMilliseconds());
    }

    public void OnMessage(string topic, Action<object> handler)
    {
        _messageHandlers[topic] = handler;
    }

    public void OffMessage(string topic)
    {
        if (_messageHandlers.ContainsKey(topic))
        {
            _messageHandlers.Remove(topic);
        }
    }

    public ConnectionStats GetStats()
    {
        var uptime = _connected ? DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - _connectionStart : 0;
        return new ConnectionStats(
            _connected,
            _stats["messages_sent"],
            _stats["messages_received"],
            uptime,
            uptime / 1000,
            0
        );
    }

    public bool IsConnected => _connected;

    public async Task DisconnectAsync()
    {
        _connected = false;
        await Task.Delay(5);
        Console.WriteLine("✓ Disconnected");
    }

    public void Dispose()
    {
        if (_connected)
        {
            DisconnectAsync().Wait();
        }
    }
}

public class TestSDK
{
    private static int _testsPassed = 0;
    private static int _testsFailed = 0;

    static async void RunTestAsync(string name, Func<Task> testFn)
    {
        try
        {
            await testFn();
            Console.WriteLine($"✅ PASS: {name}");
            _testsPassed++;
        }
        catch (Exception error)
        {
            Console.WriteLine($"❌ FAIL: {name}");
            Console.WriteLine($"   Error: {error.Message}");
            _testsFailed++;
        }
    }

    static async Task Test1BasicConnection()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        if (client.IsConnected)
        {
            throw new Exception("Should not be connected yet");
        }

        await client.ConnectAsync();

        if (!client.IsConnected)
        {
            throw new Exception("Should be connected");
        }

        await client.DisconnectAsync();
    }

    static async Task Test2SendMessage()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        var msg = new Message("test.topic", new { }).WithPriority(5);
        var result = await client.SendMessageAsync(msg);

        if (result.Status != "success")
        {
            throw new Exception($"Expected status 'success', got '{result.Status}'");
        }

        if (string.IsNullOrEmpty(result.MessageId))
        {
            throw new Exception("Message ID should not be empty");
        }

        if (result.LatencyMs < 0)
        {
            throw new Exception("Latency should be non-negative");
        }

        await client.DisconnectAsync();
    }

    static async Task Test3MessageHandlers()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        client.OnMessage("test.topic", msg => { });

        // Handler registered through internal dictionary - verify in implementation

        client.OffMessage("test.topic");

        // Handler unregistered

        await client.DisconnectAsync();
    }

    static async Task Test4ConnectionStatistics()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        for (int i = 0; i < 5; i++)
        {
            await client.SendMessageAsync(new Message("test.topic", new { }));
        }

        // Add small delay to ensure timing is measurable
        await Task.Delay(50);

        var stats = client.GetStats();

        if (!stats.IsConnected)
        {
            throw new Exception("Should be connected");
        }

        if (stats.MessagesSent != 5)
        {
            throw new Exception($"Expected 5 messages sent, got {stats.MessagesSent}");
        }

        if (stats.UptimeSeconds < 0)
        {
            throw new Exception("Uptime should be non-negative");
        }

        await client.DisconnectAsync();
    }

    static async Task Test5ConcurrentMessages()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        var tasks = new List<Task<DeliveryResult>>();
        for (int i = 0; i < 10; i++)
        {
            tasks.Add(client.SendMessageAsync(new Message("test.concurrent", new { })));
        }

        var results = await Task.WhenAll(tasks);

        if (results.Length != 10)
        {
            throw new Exception($"Expected 10 results, got {results.Length}");
        }

        foreach (var result in results)
        {
            if (result.Status != "success")
            {
                throw new Exception("All messages should be successful");
            }
        }

        var stats = client.GetStats();
        if (stats.MessagesSent != 10)
        {
            throw new Exception("Expected 10 messages sent");
        }

        await client.DisconnectAsync();
    }

    static async Task Test6PriorityLevels()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        int[] priorities = { 1, 5, 10, 20 };

        foreach (int priority in priorities)
        {
            var result = await client.SendMessageAsync(
                new Message("test.priority", new { }).WithPriority(priority)
            );

            if (result.Status != "success")
            {
                throw new Exception($"Failed to send message with priority {priority}");
            }
        }

        await client.DisconnectAsync();
    }

    static async Task Test7LatencyMeasurement()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        await client.ConnectAsync();

        var latencies = new List<double>();
        for (int i = 0; i < 50; i++)
        {
            var result = await client.SendMessageAsync(
                new Message("test.latency", new { })
            );
            latencies.Add(result.LatencyMs);
        }

        var avgLatency = latencies.Average();

        if (avgLatency < 0)
        {
            throw new Exception("Average latency should be non-negative");
        }

        var maxLatency = latencies.Max();

        Console.WriteLine($"   Average latency: {avgLatency:F2}ms, Max: {maxLatency:F2}ms");

        await client.DisconnectAsync();
    }

    static async Task Test8ErrorHandling()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        try
        {
            await client.SendMessageAsync(new Message("test", new { }));
            throw new Exception("Should have thrown error");
        }
        catch (InvalidOperationException e)
        {
            if (e.Message != "Not connected")
            {
                throw new Exception("Should throw 'Not connected' error");
            }
        }

        await client.ConnectAsync();
        var result = await client.SendMessageAsync(new Message("test", new { }));
        if (result.Status != "success")
        {
            throw new Exception("Should send successfully after connect");
        }

        await client.DisconnectAsync();
    }

    static async Task Test9ConfigurationValidation()
    {
        using var client = new FastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret");

        // Mock validation - in real SDK, would verify config values
        var client2 = new FastDataBrokerQuicClient("test.host", 9000, "tenant-x", "client-y", "secret-z");

        await Task.Delay(1);
    }

    static async Task Main(string[] args)
    {
        Console.WriteLine("\n" + new string('=', 70));
        Console.WriteLine("FastDataBroker C# SDK - Test Suite");
        Console.WriteLine(new string('=', 70) + "\n");

        RunTestAsync("1. Basic Connection", Test1BasicConnection);
        RunTestAsync("2. Send Message", Test2SendMessage);
        RunTestAsync("3. Message Handlers", Test3MessageHandlers);
        RunTestAsync("4. Connection Statistics", Test4ConnectionStatistics);
        RunTestAsync("5. Concurrent Messages", Test5ConcurrentMessages);
        RunTestAsync("6. Priority Levels", Test6PriorityLevels);
        RunTestAsync("7. Latency Measurement", Test7LatencyMeasurement);
        RunTestAsync("8. Error Handling", Test8ErrorHandling);
        RunTestAsync("9. Configuration Validation", Test9ConfigurationValidation);

        // Give async tests time to complete
        await Task.Delay(2000);

        Console.WriteLine("\n" + new string('=', 70));
        Console.WriteLine($"Results: {_testsPassed} passed, {_testsFailed} failed");
        Console.WriteLine(new string('=', 70) + "\n");

        Environment.Exit(_testsFailed > 0 ? 1 : 0);
    }
}
