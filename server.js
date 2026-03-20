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
const TICK_RATE = 60;
const BASE_SPEED = 2;
const MAX_SPEED = 14;
const MAX_PLAYERS = 10;
const POWERUP_RESPAWN_DELAY = 2000;

const NEON_COLORS = [
  '#00ff88', '#ff00ff', '#00ffff', '#ffff00',
  '#ff8800', '#88ff00', '#ff0088', '#00ffaa',
  '#ff4444', '#44ff44'
];

const PREFIXES = ['GHOST', 'CIPHER', 'NEON', 'VECTOR', 'PIXEL', 'GLITCH', 'BYTE', 'NEXUS', 'PROXY', 'SYNC'];
const SUFFIXES = ['42', '7X', '99', '3K', '77', 'AA', 'ZZ', 'Q', 'XX', '01'];

  const POWERUP_TYPES = {
    SPEED: { color: '#ffff00', duration: 5000, name: 'SPEED' },
    SHIELD: { color: '#00ffff', duration: 3000, name: 'SHIELD' },
    BOMB: { color: '#ff0000', duration: 0, name: 'BOMB' },
    GHOST: { color: '#9900ff', duration: 5000, name: 'GHOST' },
    MAGNET: { color: '#ff00ff', duration: 5000, name: 'MAGNET' },
    GROW: { color: '#00ff00', duration: 0, name: 'GROW' },
    SHRINK: { color: '#ff8800', duration: 0, name: 'SHRINK' }
  };

const POWERUP_KEYS = Object.keys(POWERUP_TYPES);

function randomName() {
  const p = PREFIXES[Math.floor(Math.random() * PREFIXES.length)];
  const s = SUFFIXES[Math.floor(Math.random() * SUFFIXES.length)];
  return `${p}_${s}`;
}

function getNextColor() {
  const used = Object.values(gameState.snakes).map(s => s.color);
  for (const c of NEON_COLORS) {
    if (!used.includes(c)) return c;
  }
  return NEON_COLORS[Math.floor(Math.random() * NEON_COLORS.length)];
}

let gameState = {
  snakes: {},
  foods: {},
  bonusFoods: [],
  powerups: [],
  tick: 0
};

let powerupRespawnTimeout = null;

function createSnake(id) {
  const startX = Math.floor(Math.random() * (COLS - 10)) + 5;
  const startY = Math.floor(Math.random() * (ROWS - 10)) + 5;
  const dirs = [{ x: 1, y: 0 }, { x: -1, y: 0 }, { x: 0, y: 1 }, { x: 0, y: -1 }];
  const d = dirs[Math.floor(Math.random() * dirs.length)];
  return {
    segments: [{ x: startX, y: startY }],
    dir: { ...d },
    nextDir: { ...d },
    color: getNextColor(),
    name: randomName(),
    score: 0,
    alive: true,
    spawned: false,
    speed: BASE_SPEED,
    frameCount: 0,
    deathReason: null,
    heldPowerup: null,
    activeEffects: {},
    superMeter: 0,
    superModeStart: null,
    ownFoodCount: 0
  };
}

function getOccupiedPositions() {
  const occupied = new Set();
  for (const s of Object.values(gameState.snakes)) {
    for (const seg of s.segments) occupied.add(`${seg.x},${seg.y}`);
  }
  for (const [id, f] of Object.entries(gameState.foods)) {
    if (f) occupied.add(`${f.x},${f.y}`);
  }
  for (const bf of gameState.bonusFoods) {
    occupied.add(`${bf.x},${bf.y}`);
  }
  for (const pu of gameState.powerups) {
    occupied.add(`${pu.x},${pu.y}`);
  }
  return occupied;
}

function spawnFood(ownerId, ownerColor, isSuper = false) {
  const occupied = getOccupiedPositions();
  let pos = null, attempts = 0;
  while (!pos && attempts < 1000) {
    const x = Math.floor(Math.random() * COLS);
    const y = Math.floor(Math.random() * ROWS);
    if (!occupied.has(`${x},${y}`)) pos = { x, y };
    attempts++;
  }
  if (!pos) return null;
  return { x: pos.x, y: pos.y, ownerId: ownerId, color: ownerColor, isSuper: isSuper, expiresAt: Date.now() + 60000 };
}

const BONUS_COLORS = ['#ffd700', '#ffffff', '#ff69b4', '#00ff99', '#ff6600', '#99ff00'];

function getBonusColor() {
  const used = Object.values(gameState.snakes).map(s => s.color);
  const available = BONUS_COLORS.filter(c => !used.includes(c));
  if (available.length === 0) return BONUS_COLORS[Math.floor(Math.random() * BONUS_COLORS.length)];
  return available[Math.floor(Math.random() * available.length)];
}

function spawnBonusFood() {
  if (gameState.bonusFoods.length >= 3) return;
  const occupied = getOccupiedPositions();
  for (const bf of gameState.bonusFoods) {
    occupied.add(`${bf.x},${bf.y}`);
  }
  let pos = null, attempts = 0;
  while (!pos && attempts < 1000) {
    const x = Math.floor(Math.random() * COLS);
    const y = Math.floor(Math.random() * ROWS);
    if (!occupied.has(`${x},${y}`)) pos = { x, y };
    attempts++;
  }
  if (!pos) return;
  const color = getBonusColor();
  const isRing = Math.random() < 0.5;
  gameState.bonusFoods.push({
    x: pos.x,
    y: pos.y,
    color: color,
    isRing: isRing,
    expiresAt: Date.now() + 60000
  });
  console.log(`[BONUS] spawned at ${pos.x},${pos.y} (${gameState.bonusFoods.length}/3)`);
}

function spawnPowerup() {
  if (gameState.powerups.length >= 3) {
    console.log(`[POWERUP] spawn blocked - already have 3 powerups`);
    return;
  }
  const occupied = getOccupiedPositions();
  console.log(`[POWERUP] occupied positions count: ${occupied.size}`);
  let pos = null, attempts = 0;
  while (!pos && attempts < 1000) {
    const x = Math.floor(Math.random() * COLS);
    const y = Math.floor(Math.random() * ROWS);
    if (!occupied.has(`${x},${y}`)) pos = { x, y };
    attempts++;
  }
  if (!pos) {
    console.log(`[POWERUP] spawn failed - no empty position found after 1000 attempts`);
    return;
  }
  
  const type = POWERUP_KEYS[Math.floor(Math.random() * POWERUP_KEYS.length)];
  console.log(`[POWERUP] selected type: ${type} from ${POWERUP_KEYS.join(', ')}`);
  const powerupData = POWERUP_TYPES[type];
  const powerup = {
    x: pos.x,
    y: pos.y,
    type: type,
    color: powerupData.color,
    expiresAt: Date.now() + 30000
  };
  gameState.powerups.push(powerup);
  console.log(`[POWERUP] ${type} spawned at ${pos.x},${pos.y} (${gameState.powerups.length}/3)`);
}

function dropPowerup(snake) {
  if (gameState.powerups.length >= 3) return;
  const head = snake.segments[0];
  const type = POWERUP_KEYS[Math.floor(Math.random() * POWERUP_KEYS.length)];
  const powerupData = POWERUP_TYPES[type];
  const powerup = {
    x: head.x,
    y: head.y,
    type: type,
    color: powerupData.color,
    expiresAt: Date.now() + 30000
  };
  gameState.powerups.push(powerup);
  console.log(`[POWERUP] ${type} dropped by ${snake.color} at ${head.x},${head.y}`);
}

function schedulePowerupRespawn() {
  if (powerupRespawnTimeout) {
    console.log(`[POWERUP] respawn already scheduled`);
    return;
  }
  console.log(`[POWERUP] scheduling respawn in ${POWERUP_RESPAWN_DELAY}ms`);
  powerupRespawnTimeout = setTimeout(() => {
    powerupRespawnTimeout = null;
    spawnPowerup();
  }, POWERUP_RESPAWN_DELAY);
}

function activatePowerup(snake, type) {
  const powerupData = POWERUP_TYPES[type];
  
  if (type === 'BOMB') {
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated BOMB`);
    const head = snake.segments[0];
    let slowed = 0;
    
    for (const other of Object.values(gameState.snakes)) {
      if (!other.alive || other === snake) continue;
      if (other.activeEffects.shield && other.activeEffects.shield > Date.now()) continue;
      const dist = Math.abs(other.segments[0].x - head.x) + Math.abs(other.segments[0].y - head.y);
      if (dist <= 5) {
        other.activeEffects.slowed = Date.now() + 3000;
        console.log(`[SLOW] ${other.color} ${other.name} slowed for 3s`);
        slowed++;
      }
    }
    
    for (const food of gameState.bonusFoods) {
      const dx = food.x - head.x;
      const dy = food.y - head.y;
      const dist = Math.abs(dx) + Math.abs(dy);
      if (dist <= 5 && dist > 0) {
        const pushX = dx === 0 ? 0 : (dx > 0 ? 1 : -1);
        const pushY = dy === 0 ? 0 : (dy > 0 ? 1 : -1);
        for (let i = 0; i < 5; i++) {
          const newX = food.x + pushX;
          const newY = food.y + pushY;
          if (newX < 0 || newX >= COLS || newY < 0 || newY >= ROWS) break;
          food.x = newX;
          food.y = newY;
        }
        food.expiresAt = Date.now() + 60000;
      }
    }
    
    for (const pu of gameState.powerups) {
      const dx = pu.x - head.x;
      const dy = pu.y - head.y;
      const dist = Math.abs(dx) + Math.abs(dy);
      if (dist <= 5 && dist > 0) {
        const pushX = dx === 0 ? 0 : (dx > 0 ? 1 : -1);
        const pushY = dy === 0 ? 0 : (dy > 0 ? 1 : -1);
        for (let i = 0; i < 5; i++) {
          const newX = pu.x + pushX;
          const newY = pu.y + pushY;
          if (newX < 0 || newX >= COLS || newY < 0 || newY >= ROWS) break;
          pu.x = newX;
          pu.y = newY;
        }
      }
    }
    
    console.log(`[POWERUP] BOMB slowed ${slowed} snakes`);
    io.emit('bombExplosion', { x: head.x, y: head.y, time: Date.now() });
    return;
  }
  
  if (type === 'SPEED') {
    const existing = snake.activeEffects.speedBoost || 0;
    snake.activeEffects.speedBoost = Math.max(existing, Date.now()) + powerupData.duration;
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated SPEED (${powerupData.duration}ms)`);
  } else if (type === 'SHIELD') {
    const existing = snake.activeEffects.shield || 0;
    snake.activeEffects.shield = Math.max(existing, Date.now()) + powerupData.duration;
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated SHIELD (${powerupData.duration}ms)`);
  } else if (type === 'GHOST') {
    const existing = snake.activeEffects.ghost || 0;
    snake.activeEffects.ghost = Math.max(existing, Date.now()) + powerupData.duration;
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated GHOST (${powerupData.duration}ms)`);
  } else if (type === 'MAGNET') {
    const existing = snake.activeEffects.magnet || 0;
    snake.activeEffects.magnet = Math.max(existing, Date.now()) + powerupData.duration;
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated MAGNET (${powerupData.duration}ms)`);
  } else if (type === 'GROW') {
    const tail = snake.segments[snake.segments.length - 1];
    for (let i = 0; i < 3; i++) {
      snake.segments.push({ x: tail.x, y: tail.y });
    }
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated GROW (+3 segments)`);
  } else if (type === 'SHRINK') {
    const minLen = 3;
    const removed = Math.min(3, snake.segments.length - minLen);
    for (let i = 0; i < removed; i++) {
      snake.segments.pop();
    }
    console.log(`[POWERUP] ${snake.color} ${snake.name} activated SHRINK (-${removed} segments)`);
  }
}

function killSnake(snake, reason, killerColor = null) {
  snake.alive = false;
  snake.deathReason = reason;
  if (killerColor) {
    console.log(`[DEATH] ${snake.color} ${snake.name} died: ${reason} hit by ${killerColor}`);
  } else {
    console.log(`[DEATH] ${snake.color} ${snake.name} died: ${reason}`);
  }
}

function broadcastState() {
  const state = {
    snakes: gameState.snakes,
    foods: gameState.foods,
    bonusFoods: gameState.bonusFoods,
    powerups: gameState.powerups,
    tick: gameState.tick
  };
  io.emit('gameState', state);
}

function tick() {
  const now = Date.now();
  
  for (const [foodId, food] of Object.entries(gameState.foods)) {
    if (food && food.expiresAt && food.expiresAt < now) {
      delete gameState.foods[foodId];
    }
  }
  
  for (let i = gameState.bonusFoods.length - 1; i >= 0; i--) {
    const bf = gameState.bonusFoods[i];
    if (bf.expiresAt < now) {
      gameState.bonusFoods.splice(i, 1);
      spawnBonusFood();
    }
  }
  
  for (let i = gameState.powerups.length - 1; i >= 0; i--) {
    const pu = gameState.powerups[i];
    if (pu.expiresAt < now) {
      gameState.powerups.splice(i, 1);
      schedulePowerupRespawn();
    }
  }
  
  const spawnedPlayers = Object.values(gameState.snakes).filter(p => p.alive && p.spawned);

  for (const p of spawnedPlayers) {
    p.frameCount++;
    const speedBoost = p.activeEffects.speedBoost && p.activeEffects.speedBoost > now;
    const slowed = p.activeEffects.slowed && p.activeEffects.slowed > now;
    let effectiveSpeed = speedBoost ? p.speed + 4 : p.speed;
    if (slowed) effectiveSpeed = Math.max(1, Math.floor(effectiveSpeed * 0.5));
    if (p.frameCount % Math.max(3, 15 - effectiveSpeed) !== 0) continue;

    p.dir = { ...p.nextDir };
    const head = p.segments[0];
    const newHead = {
      x: head.x + p.dir.x,
      y: head.y + p.dir.y
    };

    const ghostActive = p.activeEffects.ghost && p.activeEffects.ghost > now;
    const shieldActive = p.activeEffects.shield && p.activeEffects.shield > now;
    const superActive = p.activeEffects.superMode && p.activeEffects.superMode > now;

    if (superActive && p.superModeStart) {
      const elapsed = (now - p.superModeStart) / 1000;
      p.superMeter = Math.max(0, 100 - elapsed * 20);
      if (p.superMeter <= 0) {
        p.activeEffects.superMode = null;
        console.log(`[SUPER] ${p.color} ${p.name} SUPER MODE ended (meter drained)`);
      }
    }

    if (!ghostActive && !superActive && (newHead.x < 0 || newHead.x >= COLS || newHead.y < 0 || newHead.y >= ROWS)) {
      if (shieldActive) {
        let safeDir = null;
        if (newHead.x < 0) safeDir = { x: 1, y: 0 };
        else if (newHead.x >= COLS) safeDir = { x: -1, y: 0 };
        else if (newHead.y < 0) safeDir = { x: 0, y: 1 };
        else if (newHead.y >= ROWS) safeDir = { x: 0, y: -1 };
        
        if (safeDir) {
          p.nextDir = safeDir;
          p.dir = safeDir;
          console.log(`[SHIELD] ${p.color} auto-turned from wall`);
        }
        continue;
      }
      killSnake(p, 'WALL');
      dropPowerup(p);
      continue;
    }

    if (!ghostActive && !shieldActive && !superActive) {
      for (let i = 1; i < p.segments.length; i++) {
        if (newHead.x === p.segments[i].x && newHead.y === p.segments[i].y) {
          killSnake(p, 'SELF');
          dropPowerup(p);
          break;
        }
      }
      if (!p.alive) continue;
    }

    if (!ghostActive) {
      for (const other of Object.values(gameState.snakes)) {
        if (!other.alive || other === p) continue;
        for (let i = 1; i < other.segments.length; i++) {
          if (newHead.x === other.segments[i].x && newHead.y === other.segments[i].y) {
            if (shieldActive || superActive) {
              console.log(`[SHIELD/STAR] ${other.color} killed by ${p.color} ram`);
              killSnake(other, 'RAM', p.color);
              dropPowerup(other);
            } else {
              killSnake(p, 'SNAKE', other.color);
              dropPowerup(p);
            }
            break;
          }
        }
        if (!p.alive) break;
      }
    }
    if (!p.alive) continue;

    for (const other of Object.values(gameState.snakes)) {
      if (!other.alive || other === p || !other.spawned) continue;
      const otherGhost = other.activeEffects.ghost && other.activeEffects.ghost > now;
      const otherShield = other.activeEffects.shield && other.activeEffects.shield > now;
      const otherSuper = other.activeEffects.superMode && other.activeEffects.superMode > now;
      
      if (ghostActive || otherGhost) continue;
      
      const otherHead = other.segments[0];
      if (newHead.x === otherHead.x && newHead.y === otherHead.y) {
        if (superActive) {
          console.log(`[DEATH] ${other.color} ${other.name} died: HEAD_TO_HEAD (star mode ${p.color})`);
          killSnake(other, 'HEAD_TO_HEAD', p.color);
          dropPowerup(other);
        } else if (otherSuper) {
          console.log(`[DEATH] ${p.color} ${p.name} died: HEAD_TO_HEAD (star mode ${other.color})`);
          killSnake(p, 'HEAD_TO_HEAD', other.color);
          dropPowerup(p);
        } else if (shieldActive) {
          console.log(`[DEATH] ${other.color} ${other.name} died: HEAD_TO_HEAD (shielded ${p.color})`);
          killSnake(other, 'HEAD_TO_HEAD', p.color);
          dropPowerup(other);
        } else if (otherShield) {
          console.log(`[DEATH] ${p.color} ${p.name} died: HEAD_TO_HEAD (shielded ${other.color})`);
          killSnake(p, 'HEAD_TO_HEAD', other.color);
          dropPowerup(p);
        } else if (p.speed > other.speed) {
          console.log(`[DEATH] ${other.color} ${other.name} died: HEAD_TO_HEAD (${other.speed} < ${p.speed})`);
          killSnake(other, 'HEAD_TO_HEAD', p.color);
          dropPowerup(other);
        } else if (other.speed > p.speed) {
          console.log(`[DEATH] ${p.color} ${p.name} died: HEAD_TO_HEAD (${p.speed} < ${other.speed})`);
          killSnake(p, 'HEAD_TO_HEAD', other.color);
          dropPowerup(p);
        } else {
          console.log(`[DEATH] ${p.color} ${p.name} and ${other.color} ${other.name} died: HEAD_TO_HEAD (tie at ${p.speed})`);
          killSnake(p, 'HEAD_TO_HEAD', other.color);
          killSnake(other, 'HEAD_TO_HEAD', p.color);
          dropPowerup(p);
          dropPowerup(other);
        }
        break;
      }
    }
    if (!p.alive) continue;

    let grew = false;

    for (const [foodOwnerId, food] of Object.entries(gameState.foods)) {
      if (!food) continue;
      if (newHead.x === food.x && newHead.y === food.y) {
        if (shieldActive) {
          console.log(`[SHIELD] ${p.color} deflected food from ${food.color}`);
        } else if (food.color === p.color) {
          const isSuperFood = food.isSuper;
          if (isSuperFood || p.superMeter >= 100) {
            console.log(`[SUPER] ${p.color} ${p.name} activated SUPER MODE!`);
            p.activeEffects.superMode = Date.now() + 5000;
            p.activeEffects.speedBoost = Math.max(p.activeEffects.speedBoost || 0, Date.now()) + 5000;
            p.activeEffects.shield = Math.max(p.activeEffects.shield || 0, Date.now()) + 5000;
            p.activeEffects.magnet = Math.max(p.activeEffects.magnet || 0, Date.now()) + 5000;
            p.superMeter = 100;
            p.superModeStart = Date.now();
            p.ownFoodCount = 0;
          } else {
            grew = true;
            p.superMeter = Math.min(100, p.superMeter + 20);
            p.ownFoodCount++;
            console.log(`[FOOD] ${p.color} ${p.name} ate OWN food (+1 grow, meter: ${p.superMeter}%)`);
          }
        } else {
          const points = 50 * p.segments.length;
          p.score += points;
          p.superMeter = Math.min(100, p.superMeter + 10);
          console.log(`[FOOD] ${p.color} ${p.name} ate ENEMY food (+${points} points, meter: ${p.superMeter}%)`);
          delete gameState.foods[foodOwnerId];
          break;
        }
        const shouldBeSuper = p.ownFoodCount >= 5;
        gameState.foods[foodOwnerId] = spawnFood(foodOwnerId, food.color, shouldBeSuper);
        break;
      }
    }

    for (let i = gameState.bonusFoods.length - 1; i >= 0; i--) {
      const bf = gameState.bonusFoods[i];
      if (newHead.x === bf.x && newHead.y === bf.y) {
        const points = 100 * p.segments.length;
        p.score += points;
        p.superMeter = Math.min(100, p.superMeter + 10);
        console.log(`[BONUS] ${p.color} ${p.name} ate BONUS food (+${points} points, meter: ${p.superMeter}%)`);
        gameState.bonusFoods.splice(i, 1);
        spawnBonusFood();
      }
    }

    for (let i = gameState.powerups.length - 1; i >= 0; i--) {
      const pu = gameState.powerups[i];
      if (newHead.x === pu.x && newHead.y === pu.y) {
        if (shieldActive) {
          console.log(`[SHIELD] ${p.color} deflected powerup`);
        } else if (!p.heldPowerup) {
          p.heldPowerup = pu.type;
          console.log(`[POWERUP] ${p.color} ${p.name} picked up ${p.heldPowerup}`);
          gameState.powerups.splice(i, 1);
        }
      }
    }

    const magnetActive = p.activeEffects.magnet && p.activeEffects.magnet > now;
    if (magnetActive) {
      const magnetRange = Math.floor((COLS + ROWS) / 4);
      for (const [foodId, food] of Object.entries(gameState.foods)) {
        if (!food) continue;
        const dist = Math.abs(food.x - newHead.x) + Math.abs(food.y - newHead.y);
        if (dist <= magnetRange && dist > 0) {
          const dx = food.x - newHead.x;
          const dy = food.y - newHead.y;
          if (Math.abs(dx) >= Math.abs(dy) && dx !== 0) {
            food.x += dx > 0 ? -1 : 1;
          } else if (dy !== 0) {
            food.y += dy > 0 ? -1 : 1;
          }
          const newDist = Math.abs(food.x - newHead.x) + Math.abs(food.y - newHead.y);
          if (newDist <= 1) {
            const isOwnFood = food.color === p.color;
            if (isOwnFood) {
              const isSuperFood = food.isSuper;
              if (isSuperFood || p.superMeter >= 100) {
                console.log(`[SUPER] ${p.color} ${p.name} activated SUPER MODE via magnet!`);
                p.activeEffects.superMode = Date.now() + 5000;
                p.activeEffects.speedBoost = Math.max(p.activeEffects.speedBoost || 0, Date.now()) + 5000;
                p.activeEffects.shield = Math.max(p.activeEffects.shield || 0, Date.now()) + 5000;
                p.activeEffects.magnet = Math.max(p.activeEffects.magnet || 0, Date.now()) + 5000;
                p.superMeter = 100;
                p.superModeStart = Date.now();
                p.ownFoodCount = 0;
              } else {
                grew = true;
                p.superMeter = Math.min(100, p.superMeter + 20);
                p.ownFoodCount++;
              }
            } else {
              const points = 50 * p.segments.length;
              p.score += points;
              p.superMeter = Math.min(100, p.superMeter + 10);
              console.log(`[MAGNET] ${p.color} ${p.name} absorbed ENEMY food via magnet (+${points} points, meter: ${p.superMeter}%)`);
            }
            delete gameState.foods[foodId];
            const shouldBeSuper = p.ownFoodCount >= 5;
            gameState.foods[foodId] = spawnFood(foodId, food.color, shouldBeSuper);
          }
        }
      }
      for (let bi = gameState.bonusFoods.length - 1; bi >= 0; bi--) {
        const bf = gameState.bonusFoods[bi];
        const dist = Math.abs(bf.x - newHead.x) + Math.abs(bf.y - newHead.y);
        if (dist <= magnetRange && dist > 0) {
          const dx = bf.x - newHead.x;
          const dy = bf.y - newHead.y;
          if (Math.abs(dx) >= Math.abs(dy) && dx !== 0) {
            bf.x += dx > 0 ? -1 : 1;
          } else if (dy !== 0) {
            bf.y += dy > 0 ? -1 : 1;
          }
          const newDist = Math.abs(bf.x - newHead.x) + Math.abs(bf.y - newHead.y);
          if (newDist <= 1) {
            const points = 100 * p.segments.length;
            p.score += points;
            p.superMeter = Math.min(100, p.superMeter + 10);
            console.log(`[MAGNET] ${p.color} ${p.name} absorbed BONUS food via magnet (+${points} points, meter: ${p.superMeter}%)`);
            gameState.bonusFoods.splice(bi, 1);
            spawnBonusFood();
          }
        }
      }
    }

    p.segments.unshift(newHead);

    if (grew) {
      const multiplier = superActive ? 3 : 1;
      p.score += 100 * multiplier;
      if (superActive) console.log(`[SUPER] ${p.color} ${p.name} scored ${100 * multiplier} (3x)!`);
    } else {
      p.segments.pop();
    }

    if (p.activeEffects.ghost && p.activeEffects.ghost <= now) {
      const head = p.segments[0];
      if (head.x < 0 || head.x >= COLS || head.y < 0 || head.y >= ROWS) {
        killSnake(p, 'GHOST_OUT');
        dropPowerup(p);
      }
    }
  }

  gameState.tick++;
  broadcastState();
}

let tickInterval = null;
let powerupSpawnInterval = null;

function startGameLoop() {
  if (!tickInterval) {
    tickInterval = setInterval(tick, 1000 / TICK_RATE);
  }
  if (!powerupSpawnInterval) {
    powerupSpawnInterval = setInterval(() => {
      if (gameState.powerups.length < 3) {
        spawnPowerup();
      }
    }, 10000);  // Every 10 seconds
  }
  for (let i = 0; i < 3; i++) {
    spawnBonusFood();
  }
  
  setInterval(() => {
    if (gameState.bonusFoods.length < 2) {
      spawnBonusFood();
    }
  }, 8000);  // Every 8 seconds ensure 2-3 bonus food
}

io.on('connection', (socket) => {
  console.log(`[CONNECT] ${socket.id}`);

  if (Object.keys(gameState.snakes).length >= MAX_PLAYERS) {
    socket.emit('full');
    socket.disconnect();
    return;
  }

  gameState.snakes[socket.id] = createSnake(socket.id);

  socket.emit('welcome', {
    id: socket.id,
    snake: gameState.snakes[socket.id]
  });

  if (Object.keys(gameState.snakes).length >= 1 && !tickInterval) {
    spawnPowerup();
    spawnPowerup();
    startGameLoop();
  }

  broadcastState();

  socket.on('spawn', () => {
    const snake = gameState.snakes[socket.id];
    if (snake && !snake.spawned) {
      snake.spawned = true;
      console.log(`[SPAWN] ${snake.color} ${snake.name} SPAWNED`);
      if (!gameState.foods[socket.id]) {
        gameState.foods[socket.id] = spawnFood(socket.id, snake.color);
      }
      broadcastState();
    }
  });

  socket.on('input', (dirString) => {
    const p = gameState.snakes[socket.id];
    if (!p || !p.alive || !p.spawned) return;

    const opposites = { up: 'down', down: 'up', left: 'right', right: 'left' };
    const dirMap = {
      up: { x: 0, y: -1 },
      down: { x: 0, y: 1 },
      left: { x: -1, y: 0 },
      right: { x: 1, y: 0 }
    };

    if (dirMap[dirString] && dirString !== opposites[p.dir.dir]) {
      p.nextDir = dirMap[dirString];
    }
  });

  socket.on('activatePowerup', () => {
    const p = gameState.snakes[socket.id];
    if (!p || !p.alive || !p.spawned) return;
    if (p.heldPowerup) {
      activatePowerup(p, p.heldPowerup);
      p.heldPowerup = null;
      broadcastState();
    }
  });

  socket.on('ping', () => {
    socket.emit('pong');
  });

  socket.on('respawn', () => {
    if (gameState.snakes[socket.id]) {
      const oldSnake = gameState.snakes[socket.id];
      const newSnake = createSnake(socket.id);
      newSnake.color = oldSnake.color;
      newSnake.name = oldSnake.name;
      newSnake.score = oldSnake.score;
      newSnake.spawned = true;
      gameState.snakes[socket.id] = newSnake;
      gameState.foods[socket.id] = spawnFood(socket.id, newSnake.color);

      console.log(`[RESPAWN] ${newSnake.color} ${newSnake.name} respawned`);
      broadcastState();
    }
  });

  socket.on('disconnect', () => {
    const snake = gameState.snakes[socket.id];
    if (snake) {
      console.log(`[DISCONNECT] ${snake.color} ${snake.name}`);
    }
    delete gameState.snakes[socket.id];
    delete gameState.foods[socket.id];
    if (Object.keys(gameState.snakes).length === 0) {
      clearInterval(tickInterval);
      tickInterval = null;
      clearInterval(powerupSpawnInterval);
      powerupSpawnInterval = null;
      gameState.powerups = [];
    }
    broadcastState();
  });
});

const PORT = process.env.PORT || 3000;
server.listen(PORT, () => {
  console.log(`[SERVER] CYBER_SNAKE running on port ${PORT}`);
});