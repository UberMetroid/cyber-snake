const express = require('express');
const http = require('http');
const { Server } = require('socket.io');
const path = require('path');

const app = express();
const server = http.createServer(app);
const io = new Server(server, {
  cors: { origin: '*' }
});

app.use(express.static(path.join(__dirname, 'public')));

const GRID = 20;
const COLS = 30;
const ROWS = 30;
const TICK_RATE = 15;
const BASE_SPEED = 8;

const PLAYER_COLORS = [
  '#00ff88', '#ff00ff', '#00ffff', '#ffff00',
  '#ff8800', '#88ff00', '#ff0088', '#00ffaa'
];

let players = {};
let food = null;
let gameLoop = null;
let tickInterval = null;

function getNextColor() {
  const usedColors = Object.values(players).map(p => p.color);
  for (const c of PLAYER_COLORS) {
    if (!usedColors.includes(c)) return c;
  }
  return PLAYER_COLORS[Math.floor(Math.random() * PLAYER_COLORS.length)];
}

function createSnake(id) {
  const startX = Math.floor(Math.random() * (COLS - 10)) + 5;
  const startY = Math.floor(Math.random() * (ROWS - 10)) + 5;
  const dirs = [{ x: 1, y: 0 }, { x: -1, y: 0 }, { x: 0, y: 1 }, { x: 0, y: -1 }];
  const dir = dirs[Math.floor(Math.random() * dirs.length)];
  return {
    id,
    segments: [{ x: startX, y: startY }],
    dir: { ...dir },
    nextDir: { ...dir },
    color: getNextColor(),
    score: 0,
    alive: true,
    speed: BASE_SPEED,
    frameCount: 0
  };
}

function spawnFood() {
  const occupied = new Set();
  for (const p of Object.values(players)) {
    for (const s of p.segments) {
      occupied.add(`${s.x},${s.y}`);
    }
  }
  let pos = null;
  let attempts = 0;
  while (!pos && attempts < 1000) {
    const x = Math.floor(Math.random() * COLS);
    const y = Math.floor(Math.random() * ROWS);
    if (!occupied.has(`${x},${y}`)) {
      pos = { x, y };
    }
    attempts++;
  }
  food = pos;
}

function resetGame() {
  for (const id in players) {
    players[id] = createSnake(id);
  }
  spawnFood();
  broadcastState();
}

function broadcastState() {
  const state = {
    players: Object.values(players).map(p => ({
      id: p.id,
      segments: p.segments,
      color: p.color,
      score: p.score,
      alive: p.alive,
      dir: p.dir
    })),
    food
  };
  io.emit('gameState', state);
}

function tick() {
  const alivePlayers = Object.values(players).filter(p => p.alive);
  
  for (const p of alivePlayers) {
    p.frameCount++;
    if (p.frameCount % Math.max(3, 15 - p.speed) !== 0) continue;

    p.dir = { ...p.nextDir };
    const head = p.segments[0];
    const newHead = {
      x: head.x + p.dir.x,
      y: head.y + p.dir.y
    };

    if (newHead.x < 0 || newHead.x >= COLS || newHead.y < 0 || newHead.y >= ROWS) {
      p.alive = false;
      continue;
    }

    for (const other of alivePlayers) {
      for (let i = 0; i < other.segments.length; i++) {
        if (i === 0 && other === p) continue;
        if (newHead.x === other.segments[i].x && newHead.y === other.segments[i].y) {
          p.alive = false;
          break;
        }
      }
      if (!p.alive) break;
    }
    if (!p.alive) continue;

    p.segments.unshift(newHead);

    if (food && newHead.x === food.x && newHead.y === food.y) {
      p.score += 100;
      p.speed = Math.min(12, BASE_SPEED + Math.floor(p.score / 500));
      spawnFood();
    } else {
      p.segments.pop();
    }
  }

  const stillAlive = Object.values(players).filter(p => p.alive);
  if (stillAlive.length <= 1 && Object.keys(players).length > 0) {
    setTimeout(resetGame, 2000);
  }

  broadcastState();
}

io.on('connection', (socket) => {
  console.log(`Player connected: ${socket.id}`);
  
  players[socket.id] = createSnake(socket.id);
  
  socket.emit('welcome', {
    id: socket.id,
    grid: { GRID, COLS, ROWS }
  });

  if (Object.keys(players).length === 1 && !tickInterval) {
    tickInterval = setInterval(tick, 1000 / TICK_RATE);
  }

  broadcastState();

  socket.on('input', (dir) => {
    const p = players[socket.id];
    if (!p || !p.alive) return;
    
    const opposites = {
      up: 'down', down: 'up', left: 'right', right: 'left'
    };
    
    if (opposites[dir] !== p.dir.dir || !p.dir.dir) {
      const dirMap = {
        up: { x: 0, y: -1 },
        down: { x: 0, y: 1 },
        left: { x: -1, y: 0 },
        right: { x: 1, y: 0 }
      };
      if (dirMap[dir] && 
          !(dirMap[dir].x === -p.dir.x && dirMap[dir].y === -p.dir.y)) {
        p.nextDir = dirMap[dir];
        p.dir.dir = dir;
      }
    }
  });

  socket.on('respawn', () => {
    if (players[socket.id] && !players[socket.id].alive) {
      players[socket.id] = createSnake(socket.id);
      broadcastState();
    }
  });

  socket.on('disconnect', () => {
    console.log(`Player disconnected: ${socket.id}`);
    delete players[socket.id];
    if (Object.keys(players).length === 0) {
      clearInterval(tickInterval);
      tickInterval = null;
    }
    broadcastState();
  });
});

const PORT = process.env.PORT || 3000;
server.listen(PORT, () => {
  console.log(`CYBER_SNAKE server running on port ${PORT}`);
});