const ipc = require("node-ipc");

const CUSTOM_EVENT = "app.message";

ipc.config.id = "bridge";
ipc.config.retry = 1500;
ipc.serve();

ipc.server.on(CUSTOM_EVENT, (msg, socket) => {
  ipc.server.emit(socket, CUSTOM_EVENT, `${msg} world!`);
});

ipc.server.on("connect", (socket) => {
  const timer = setInterval(() => {
    ipc.server.emit(socket, CUSTOM_EVENT, "tick");
  }, 10000);
  socket.on("end", () => {
    clearInterval(timer);
  });
});

ipc.server.start();

// js ipc client
ipc.connectTo("bridge", () => {
  ipc.of.bridge.on("connect", () => {
    ipc.of.bridge.emit(CUSTOM_EVENT, "hello");
  });
  ipc.of.bridge.on("disconnect", () => {
    ipc.log("disconnected from world");
  });
  ipc.of.bridge.on(CUSTOM_EVENT, (msg) => {
    ipc.log("got a message from world : ", msg);
  });
});
