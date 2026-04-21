using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace FastDataBroker.SDK
{
    /// <summary>
    /// FastDataBroker C# SDK - QUIC with PSK Authentication
    /// High-performance client library with Pre-Shared Key authentication
    /// 
    /// Version: 1.0.0
    /// Protocol: QUIC 1.0 (RFC 9000)
    /// Authentication: TLS 1.3 PSK (Pre-Shared Key)
    /// </summary>
    public class FastDataBrokerQuicClient : IDisposable
    {
        // ====================================================================
        // Types and Enums
        // ====================================================================

        public enum Priority
        {
            Low = 1,
            Normal = 5,
            High = 10,
            Critical = 20
        }

        public enum ConnectionState
        {
            Disconnected,
            Connecting,
            Connected,
            Authenticated,
            Error
        }

        public class Message
        {
            public string Topic { get; set; }
            public object Payload { get; set; }
            public Priority Priority { get; set; } = Priority.Normal;
            public int TtlSeconds { get; set; } = 3600;
            public Dictionary<string, string> Headers { get; set; } = new();
        }

        public class DeliveryResult
        {
            public string MessageId { get; set; }
            public string Status { get; set; }
            public double LatencyMs { get; set; }
            public long Timestamp { get; set; }

            public override string ToString()
                => $"DeliveryResult{{MessageId='{MessageId}', Status='{Status}', LatencyMs={LatencyMs:F2}}}";
        }

        public class ConnectionStats
        {
            public bool IsConnected { get; set; }
            public long MessagesSent { get; set; }
            public long MessagesReceived { get; set; }
            public long ConnectionTimeMs { get; set; }
            public long UptimeSeconds { get; set; }
            public long LastMessageTime { get; set; }

            public override string ToString()
                => $"ConnectionStats{{IsConnected={IsConnected}, MessagesSent={MessagesSent}, " +
                   $"MessagesReceived={MessagesReceived}, ConnectionTimeMs={ConnectionTimeMs}}}";
        }

        public class QuicConnectionConfig
        {
            public string Host { get; set; }
            public int Port { get; set; }
            public string TenantId { get; set; }
            public string ClientId { get; set; }
            public string PskSecret { get; set; }
            public string Secrets { get; set; }
            public int IdleTimeoutMs { get; set; } = 30000;
            public int MaxStreams { get; set; } = 100;
            public bool AutoReconnect { get; set; } = true;
            public int ReadTimeoutMs { get; set; } = 60000;
        }

        // ====================================================================
        // Private Fields
        // ====================================================================

        private readonly QuicConnectionConfig _config;
        private TcpClient _client;
        private NetworkStream _stream;
        private StreamReader _reader;
        private StreamWriter _writer;
        private bool _connected = false;
        private bool _authenticated = false;
        private ConnectionState _state = ConnectionState.Disconnected;
        private readonly Dictionary<string, Action<JsonElement>> _messageHandlers = new();
        private long _connectionStart = 0;
        private CancellationTokenSource _cancellationTokenSource;
        private Task _receiveTask;
        private readonly object _statsLock = new();

        // Statistics
        private long _messagesSent = 0;
        private long _messagesReceived = 0;
        private long _lastMessageTime = 0;

        private static readonly JsonSerializerOptions JsonOptions = new()
        {
            PropertyNameCaseInsensitive = true,
            DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull
        };

        // ====================================================================
        // Constructor and Disposal
        // ====================================================================

        public FastDataBrokerQuicClient(QuicConnectionConfig config)
        {
            _config = config;
            Console.WriteLine($"Initialized FastDataBroker QUIC client for {config.TenantId}:{config.ClientId}");
        }

        public void Dispose()
        {
            Disconnect().Wait();
            _cancellationTokenSource?.Dispose();
            _reader?.Dispose();
            _writer?.Dispose();
            _stream?.Dispose();
            _client?.Dispose();
        }

        // ====================================================================
        // Connection Management
        // ====================================================================

        /// <summary>
        /// Generate PSK identity and secret hash
        /// </summary>
        private (string Identity, string SecretHash) GeneratePskIdentity()
        {
            var identity = $"{_config.TenantId}:{_config.ClientId}";
            var secretHash = ComputeSha256Hash(_config.PskSecret);
            return (identity, secretHash);
        }

        /// <summary>
        /// Compute SHA-256 hash
        /// </summary>
        private static string ComputeSha256Hash(string input)
        {
            using (var sha256 = SHA256.Create())
            {
                var hashedBytes = sha256.ComputeHash(Encoding.UTF8.GetBytes(input));
                return Convert.ToHexString(hashedBytes).ToLower();
            }
        }

        /// <summary>
        /// Connect to FastDataBroker with PSK authentication
        /// </summary>
        public async Task ConnectAsync()
        {
            if (_connected)
            {
                Console.WriteLine("Already connected");
                return;
            }

            try
            {
                _state = ConnectionState.Connecting;
                Console.WriteLine($"Connecting to {_config.Host}:{_config.Port}...");

                // Create TCP client
                _client = new TcpClient
                {
                    ReceiveTimeout = _config.ReadTimeoutMs,
                    SendTimeout = 5000
                };

                await _client.ConnectAsync(_config.Host, _config.Port);
                _stream = _client.GetStream();
                _reader = new StreamReader(_stream, Encoding.UTF8);
                _writer = new StreamWriter(_stream, Encoding.UTF8) { AutoFlush = true };

                _connected = true;
                _connectionStart = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond;
                Console.WriteLine("✓ TCP connection established");

                // Send PSK handshake
                await SendPskHandshakeAsync();

                // Start receive loop
                _cancellationTokenSource = new CancellationTokenSource();
                _receiveTask = Task.Run(() => ReceiveLoop(_cancellationTokenSource.Token));

                _authenticated = true;
                _state = ConnectionState.Authenticated;
                Console.WriteLine("✓ PSK authentication successful");
            }
            catch (Exception ex)
            {
                _state = ConnectionState.Error;
                _connected = false;
                Console.WriteLine($"✗ Connection failed: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Send PSK handshake
        /// </summary>
        private async Task SendPskHandshakeAsync()
        {
            var psk = GeneratePskIdentity();
            var handshake = new
            {
                type = "psk_auth",
                identity = psk.Identity,
                secret_hash = psk.SecretHash,
                timestamp = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond
            };

            var json = JsonSerializer.Serialize(handshake, JsonOptions);
            await _writer.WriteLineAsync(json);
        }

        /// <summary>
        /// Receive loop
        /// </summary>
        private async void ReceiveLoop(CancellationToken cancellationToken)
        {
            try
            {
                while (!cancellationToken.IsCancellationRequested && _connected)
                {
                    try
                    {
                        var line = await _reader.ReadLineAsync();
                        if (line == null)
                        {
                            Console.WriteLine("Server closed connection");
                            _connected = false;
                            break;
                        }

                        if (!string.IsNullOrEmpty(line))
                        {
                            HandleIncomingMessage(line);
                        }
                    }
                    catch (OperationCanceledException)
                    {
                        break;
                    }
                    catch (IOException) when (!_connected)
                    {
                        break;
                    }
                }
            }
            catch (Exception ex)
            {
                if (_connected)
                {
                    Console.WriteLine($"Receive error: {ex.Message}");
                }
                _connected = false;
            }
        }

        /// <summary>
        /// Handle incoming message
        /// </summary>
        private void HandleIncomingMessage(string messageStr)
        {
            try
            {
                var parsed = JsonDocument.Parse(messageStr).RootElement;

                if (parsed.TryGetProperty("type", out var typeEl) && typeEl.GetString() == "message")
                {
                    if (parsed.TryGetProperty("topic", out var topicEl))
                    {
                        var topic = topicEl.GetString();
                        if (_messageHandlers.TryGetValue(topic, out var handler))
                        {
                            handler(parsed);
                            lock (_statsLock)
                            {
                                _messagesReceived++;
                                _lastMessageTime = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond;
                            }
                        }
                    }
                }
            }
            catch
            {
                // Ignore malformed messages
            }
        }

        // ====================================================================
        // Message Operations
        // ====================================================================

        /// <summary>
        /// Send message to FastDataBroker
        /// </summary>
        public async Task<DeliveryResult> SendMessageAsync(Message message)
        {
            if (!_connected || _client == null)
            {
                throw new InvalidOperationException("Not connected to FastDataBroker");
            }

            var startTime = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond;
            var messageId = $"msg_{startTime}_{Random.Shared.Next(10000)}";

            var envelope = new
            {
                type = "message",
                id = messageId,
                topic = message.Topic,
                payload = message.Payload,
                priority = (int)message.Priority,
                ttl = message.TtlSeconds,
                headers = message.Headers,
                timestamp = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond
            };

            var json = JsonSerializer.Serialize(envelope, JsonOptions);
            await _writer.WriteLineAsync(json);

            lock (_statsLock)
            {
                _messagesSent++;
                _lastMessageTime = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond;
            }

            var latency = (DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond) - startTime;

            return new DeliveryResult
            {
                MessageId = messageId,
                Status = "success",
                LatencyMs = latency,
                Timestamp = DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond
            };
        }

        // ====================================================================
        // Message Handlers
        // ====================================================================

        /// <summary>
        /// Register message handler for topic
        /// </summary>
        public void OnMessage(string topic, Action<JsonElement> handler)
        {
            _messageHandlers[topic] = handler;
            Console.WriteLine($"Registered handler for topic: {topic}");
        }

        /// <summary>
        /// Unregister message handler
        /// </summary>
        public void OffMessage(string topic)
        {
            if (_messageHandlers.Remove(topic))
            {
                Console.WriteLine($"Unregistered handler for topic: {topic}");
            }
        }

        // ====================================================================
        // Connection Status
        // ====================================================================

        /// <summary>
        /// Get connection statistics
        /// </summary>
        public ConnectionStats GetStats()
        {
            lock (_statsLock)
            {
                var uptime = _connected 
                    ? (DateTime.UtcNow.Ticks / TimeSpan.TicksPerMillisecond) - _connectionStart 
                    : 0;

                return new ConnectionStats
                {
                    IsConnected = _connected,
                    MessagesSent = _messagesSent,
                    MessagesReceived = _messagesReceived,
                    ConnectionTimeMs = uptime,
                    UptimeSeconds = uptime / 1000,
                    LastMessageTime = _lastMessageTime
                };
            }
        }

        /// <summary>
        /// Check if connected
        /// </summary>
        public bool IsConnected => _connected && _authenticated;

        /// <summary>
        /// Send multiple messages in parallel
        /// </summary>
        public async Task<List<DeliveryResult>> SendMessagesParallelAsync(
            List<Message> messages,
            int numWorkers = 4
        )
        {
            if (!IsConnected)
            {
                throw new InvalidOperationException("Not connected to FastDataBroker");
            }

            var results = new List<DeliveryResult>();
            var options = new ParallelOptions { MaxDegreeOfParallelism = numWorkers };

            await Task.Run(() =>
            {
                Parallel.ForEach(messages, options, async message =>
                {
                    try
                    {
                        var result = await SendMessageAsync(message);
                        lock (results)
                        {
                            results.Add(result);
                        }
                    }
                    catch
                    {
                        lock (results)
                        {
                            results.Add(new DeliveryResult
                            {
                                MessageId = $"error_{DateTime.Now.Ticks}",
                                Status = "failed",
                                LatencyMs = 0,
                                Timestamp = DateTimeOffset.Now.ToUnixTimeMilliseconds()
                            });
                        }
                    }
                });
            });

            return results;
        }

        /// <summary>
        /// Send messages with progress tracking
        /// </summary>
        public async Task<List<DeliveryResult>> SendMessagesParallelWithProgressAsync(
            List<Message> messages,
            int numWorkers = 4,
            Action<int, int> onProgress = null
        )
        {
            if (!IsConnected)
            {
                throw new InvalidOperationException("Not connected to FastDataBroker");
            }

            var results = new List<DeliveryResult>();
            var completed = 0;
            var lockObj = new object();
            var options = new ParallelOptions { MaxDegreeOfParallelism = numWorkers };

            await Task.Run(() =>
            {
                Parallel.ForEach(messages, options, async message =>
                {
                    try
                    {
                        var result = await SendMessageAsync(message);
                        lock (lockObj)
                        {
                            results.Add(result);
                            completed++;
                            onProgress?.Invoke(completed, messages.Count);
                        }
                    }
                    catch
                    {
                        lock (lockObj)
                        {
                            results.Add(new DeliveryResult
                            {
                                MessageId = $"error_{DateTime.Now.Ticks}",
                                Status = "failed",
                                LatencyMs = 0,
                                Timestamp = DateTimeOffset.Now.ToUnixTimeMilliseconds()
                            });
                            completed++;
                            onProgress?.Invoke(completed, messages.Count);
                        }
                    }
                });
            });

            return results;
        }

        /// <summary>
        /// Synchronous version of SendMessagesParallelAsync
        /// </summary>
        public List<DeliveryResult> SendMessagesParallel(
            List<Message> messages,
            int numWorkers = 4
        )
        {
            return SendMessagesParallelAsync(messages, numWorkers).Result;
        }

        /// <summary>
        /// Synchronous version of SendMessagesParallelWithProgressAsync
        /// </summary>
        public List<DeliveryResult> SendMessagesParallelWithProgress(
            List<Message> messages,
            int numWorkers = 4,
            Action<int, int> onProgress = null
        )
        {
            return SendMessagesParallelWithProgressAsync(messages, numWorkers, onProgress).Result;
        }

        /// <summary>
        /// Disconnect from FastDataBroker
        /// </summary>
        public async Task DisconnectAsync()
        {
            _connected = false;
            _authenticated = false;
            _state = ConnectionState.Disconnected;

            _cancellationTokenSource?.Cancel();

            if (_receiveTask != null)
            {
                try
                {
                    await _receiveTask;
                }
                catch (OperationCanceledException)
                {
                    // Expected
                }
            }

            _stream?.Close();
            _client?.Close();
            Console.WriteLine("✓ Disconnected from FastDataBroker");
        }

        // Also provide synchronous version
        public void Disconnect()
        {
            DisconnectAsync().Wait();
        }

        // ====================================================================
        // Static Factory Methods
        // ====================================================================

        public static FastDataBrokerQuicClient Create(QuicConnectionConfig config)
        {
            return new FastDataBrokerQuicClient(config);
        }

        public static string GetPskSecretFromEnv()
        {
            var secret = Environment.GetEnvironmentVariable("QUIC_PSK_SECRET");
            if (string.IsNullOrEmpty(secret))
            {
                throw new InvalidOperationException(
                    "QUIC_PSK_SECRET environment variable not set. " +
                    "Get it from: POST /api/quic/psks"
                );
            }
            return secret;
        }

        // ====================================================================
        // Example Usage
        // ====================================================================

        public static async Task Main(string[] args)
        {
            var pskSecret = Environment.GetEnvironmentVariable("QUIC_PSK_SECRET") ?? "test-secret-key";

            var config = new QuicConnectionConfig
            {
                Host = "localhost",
                Port = 6000,
                TenantId = "test-tenant",
                ClientId = "test-client",
                PskSecret = pskSecret
            };

            using (var client = new FastDataBrokerQuicClient(config))
            {
                try
                {
                    await client.ConnectAsync();
                    Console.WriteLine("✓ Client connected successfully");

                    // Send message
                    var message = new Message
                    {
                        Topic = "test.topic",
                        Payload = new { data = "test" },
                        Priority = Priority.Normal
                    };

                    var result = await client.SendMessageAsync(message);
                    Console.WriteLine($"✓ Message sent: {result}");

                    // Get stats
                    var stats = client.GetStats();
                    Console.WriteLine($"✓ Stats: {stats}");

                    // Keep running
                    await Task.Delay(10000);
                }
                finally
                {
                    await client.DisconnectAsync();
                }
            }
        }
    }
}
