using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;
using FastDataBroker;

namespace FastDataBroker.Tests
{
    public class FastDataBrokerSDKTests
    {
        [Fact]
        public void TestVersion()
        {
            Assert.Equal("0.1.12", FastDataBrokerSDK.Version);
        }

        [Fact]
        public void TestPriorityEnum()
        {
            Assert.Equal((byte)50, (byte)FastDataBrokerSDK.Priority.Deferred);
            Assert.Equal((byte)100, (byte)FastDataBrokerSDK.Priority.Normal);
            Assert.Equal((byte)150, (byte)FastDataBrokerSDK.Priority.High);
            Assert.Equal((byte)200, (byte)FastDataBrokerSDK.Priority.Urgent);
            Assert.Equal((byte)255, (byte)FastDataBrokerSDK.Priority.Critical);
        }

        [Fact]
        public void TestNotificationChannelEnum()
        {
            Assert.Equal(FastDataBrokerSDK.NotificationChannel.Email, FastDataBrokerSDK.NotificationChannel.Email);
            Assert.Equal(FastDataBrokerSDK.NotificationChannel.WebSocket, FastDataBrokerSDK.NotificationChannel.WebSocket);
            Assert.Equal(FastDataBrokerSDK.NotificationChannel.Push, FastDataBrokerSDK.NotificationChannel.Push);
            Assert.Equal(FastDataBrokerSDK.NotificationChannel.Webhook, FastDataBrokerSDK.NotificationChannel.Webhook);
        }

        [Fact]
        public void TestPushPlatformEnum()
        {
            Assert.Equal(FastDataBrokerSDK.PushPlatform.Firebase, FastDataBrokerSDK.PushPlatform.Firebase);
            Assert.Equal(FastDataBrokerSDK.PushPlatform.APNs, FastDataBrokerSDK.PushPlatform.APNs);
            Assert.Equal(FastDataBrokerSDK.PushPlatform.FCM, FastDataBrokerSDK.PushPlatform.FCM);
            Assert.Equal(FastDataBrokerSDK.PushPlatform.WebPush, FastDataBrokerSDK.PushPlatform.WebPush);
        }
    }

    public class MessageTests
    {
        [Fact]
        public void TestMessageCreation()
        {
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "user-1",
                RecipientIds = new List<string> { "user-2" },
                Subject = "Test Message",
                Content = System.Text.Encoding.UTF8.GetBytes("Hello")
            };

            Assert.Equal("user-1", message.SenderId);
            Assert.Single(message.RecipientIds);
            Assert.Equal("user-2", message.RecipientIds[0]);
            Assert.Equal("Test Message", message.Subject);
        }

        [Fact]
        public void TestMessageWithConstructor()
        {
            var content = System.Text.Encoding.UTF8.GetBytes("Test Content");
            var recipients = new List<string> { "user-1", "user-2" };
            
            var message = new FastDataBrokerSDK.Message("sender-1", recipients, "Subject", content);

            Assert.Equal("sender-1", message.SenderId);
            Assert.Equal(2, message.RecipientIds.Count);
            Assert.Equal("Subject", message.Subject);
            Assert.NotEmpty(message.Content);
        }

        [Fact]
        public void TestMessageDefaultValues()
        {
            var message = new FastDataBrokerSDK.Message();

            Assert.Null(message.SenderId);
            Assert.NotNull(message.RecipientIds);
            Assert.Empty(message.RecipientIds);
            Assert.Equal(FastDataBrokerSDK.Priority.Normal, message.Priority);
            Assert.NotNull(message.Tags);
            Assert.Empty(message.Tags);
            Assert.False(message.RequireConfirm);
        }

        [Fact]
        public void TestMessageTags()
        {
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "user-1",
                Tags = new Dictionary<string, string>
                {
                    { "priority", "high" },
                    { "region", "us-east" }
                }
            };

            Assert.Equal(2, message.Tags.Count);
            Assert.Equal("high", message.Tags["priority"]);
            Assert.Equal("us-east", message.Tags["region"]);
        }

        [Fact]
        public void TestMessagePriority()
        {
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "user-1",
                Priority = FastDataBrokerSDK.Priority.Critical
            };

            Assert.Equal(FastDataBrokerSDK.Priority.Critical, message.Priority);
        }

        [Fact]
        public void TestMessageTTL()
        {
            var message = new FastDataBrokerSDK.Message
            {
                SenderId = "user-1",
                TTLSeconds = 3600
            };

            Assert.Equal(3600, message.TTLSeconds);
        }
    }

    public class DeliveryResultTests
    {
        [Fact]
        public void TestDeliveryResultCreation()
        {
            var result = new FastDataBrokerSDK.DeliveryResult
            {
                MessageId = "msg-1",
                Status = "success",
                DeliveredChannels = 2
            };

            Assert.Equal("msg-1", result.MessageId);
            Assert.Equal("success", result.Status);
            Assert.Equal(2, result.DeliveredChannels);
        }

        [Fact]
        public void TestDeliveryResultDetails()
        {
            var result = new FastDataBrokerSDK.DeliveryResult
            {
                MessageId = "msg-1",
                Details = new Dictionary<string, object>
                {
                    { "email", "delivered" },
                    { "push", "pending" }
                }
            };

            Assert.Equal(2, result.Details.Count);
            Assert.Equal("delivered", result.Details["email"]);
        }
    }

    public class ClientTests
    {
        [Fact]
        public void TestClientInitialization()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                Assert.False(client.IsConnected);
            }
        }

        [Fact]
        public async Task TestClientConnectAsync()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                var result = await client.ConnectAsync();
                Assert.True(result);
                Assert.True(client.IsConnected);
            }
        }

        [Fact]
        public async Task TestSendMessageAsyncThrowsIfNotConnected()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                var message = new FastDataBrokerSDK.Message { SenderId = "user-1" };
                
                await Assert.ThrowsAsync<InvalidOperationException>(async () =>
                {
                    await client.SendMessageAsync(message);
                });
            }
        }

        [Fact]
        public async Task TestSendMessageAsync()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                
                var message = new FastDataBrokerSDK.Message
                {
                    SenderId = "user-1",
                    RecipientIds = new List<string> { "user-2" },
                    Subject = "Test",
                    Content = System.Text.Encoding.UTF8.GetBytes("Hello")
                };

                var result = await client.SendMessageAsync(message);

                Assert.NotNull(result);
                Assert.NotNull(result.MessageId);
                Assert.Equal("success", result.Status);
            }
        }

        [Fact]
        public async Task TestSendMessageThrowsIfNotConnected()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                var message = new FastDataBrokerSDK.Message { SenderId = "user-1" };
                
                Assert.Throws<InvalidOperationException>(() =>
                {
                    client.SendMessage(message);
                });
            }
        }

        [Fact]
        public async Task TestSendMessage()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                
                var message = new FastDataBrokerSDK.Message
                {
                    SenderId = "user-1",
                    RecipientIds = new List<string> { "user-2" }
                };

                var result = client.SendMessage(message);

                Assert.NotNull(result);
                Assert.Equal("success", result.Status);
            }
        }

        [Fact]
        public async Task TestRegisterWebSocketClient()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                
                var registered = client.RegisterWebSocketClient("client-1", "user-1");
                Assert.True(registered);
            }
        }

        [Fact]
        public async Task TestUnregisterWebSocketClient()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                
                client.RegisterWebSocketClient("client-1", "user-1");
                var unregistered = client.UnregisterWebSocketClient("client-1");
                
                Assert.True(unregistered);
            }
        }

        [Fact]
        public async Task TestRegisterWebhook()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                
                var webhookConfig = new FastDataBrokerSDK.WebhookConfig
                {
                    Url = "https://example.com/webhook",
                    Retries = 3,
                    TimeoutMs = 30000
                };

                var registered = client.RegisterWebhook(
                    FastDataBrokerSDK.NotificationChannel.Webhook,
                    webhookConfig
                );

                Assert.True(registered);
            }
        }

        [Fact]
        public async Task TestDisconnect()
        {
            using (var client = new FastDataBrokerSDK.Client("localhost", 6000))
            {
                await client.ConnectAsync();
                Assert.True(client.IsConnected);
                
                client.Disconnect();
                Assert.False(client.IsConnected);
            }
        }

        [Fact]
        public void TestClientDispose()
        {
            var client = new FastDataBrokerSDK.Client("localhost", 6000);
            client.Dispose();
            Assert.False(client.IsConnected);
        }
    }

    public class WebSocketClientInfoTests
    {
        [Fact]
        public void TestWebSocketClientInfoCreation()
        {
            var clientInfo = new FastDataBrokerSDK.WebSocketClientInfo
            {
                ClientId = "client-1",
                UserId = "user-1",
                ConnectedAt = DateTime.UtcNow
            };

            Assert.Equal("client-1", clientInfo.ClientId);
            Assert.Equal("user-1", clientInfo.UserId);
            Assert.NotEqual(default(DateTime), clientInfo.ConnectedAt);
        }
    }

    public class WebhookConfigTests
    {
        [Fact]
        public void TestWebhookConfigCreation()
        {
            var config = new FastDataBrokerSDK.WebhookConfig
            {
                Url = "https://example.com/webhook",
                Retries = 5,
                TimeoutMs = 60000,
                VerifySSL = true
            };

            Assert.Equal("https://example.com/webhook", config.Url);
            Assert.Equal(5, config.Retries);
            Assert.Equal(60000, config.TimeoutMs);
            Assert.True(config.VerifySSL);
        }

        [Fact]
        public void TestWebhookConfigDefaults()
        {
            var config = new FastDataBrokerSDK.WebhookConfig();

            Assert.Equal(3, config.Retries);
            Assert.Equal(30000, config.TimeoutMs);
            Assert.True(config.VerifySSL);
        }

        [Fact]
        public void TestWebhookConfigHeaders()
        {
            var config = new FastDataBrokerSDK.WebhookConfig
            {
                Url = "https://example.com/webhook",
                Headers = new Dictionary<string, string>
                {
                    { "Authorization", "Bearer token" },
                    { "X-Custom", "value" }
                }
            };

            Assert.Equal(2, config.Headers.Count);
            Assert.Equal("Bearer token", config.Headers["Authorization"]);
        }
    }
}
