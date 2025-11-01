#!/usr/bin/env node
const http = require('http');
const fs = require('fs');
const path = require('path');

const DEFAULT_PORT = 4173;
const rootDir = path.resolve(process.argv[2] || 'dist');
const port = Number(process.argv[3] || process.env.PORT || DEFAULT_PORT);

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.ttf': 'font/ttf',
  '.wasm': 'application/wasm',
  '.pdf': 'application/pdf',
};

function serveFile(filePath, res) {
  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';
  res.writeHead(200, {
    'Content-Type': contentType,
    'Cache-Control': 'no-cache',
  });
  const stream = fs.createReadStream(filePath);
  stream.on('error', (err) => {
    console.error('Error reading', filePath, err);
    if (!res.headersSent) {
      res.writeHead(500, { 'Content-Type': 'text/plain; charset=utf-8' });
    }
    res.end('Internal server error');
  });
  stream.pipe(res);
}

const server = http.createServer((req, res) => {
  const urlPath = decodeURIComponent((req.url || '/').split('?')[0]);
  let relativePath = urlPath === '/' ? '/index.html' : urlPath;
  let requestedPath = path.join(rootDir, relativePath);

  fs.stat(requestedPath, (err, stats) => {
    if (!err && stats.isDirectory()) {
      requestedPath = path.join(requestedPath, 'index.html');
    }

    fs.stat(requestedPath, (statErr, statInfo) => {
      if (!statErr && statInfo.isFile()) {
        serveFile(requestedPath, res);
        return;
      }

      const fallbackPath = path.join(rootDir, 'index.html');
      fs.stat(fallbackPath, (fallbackErr, fallbackInfo) => {
        if (!fallbackErr && fallbackInfo.isFile()) {
          serveFile(fallbackPath, res);
        } else {
          res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
          res.end('Not found');
        }
      });
    });
  });
});

server.listen(port, '127.0.0.1', () => {
  console.log(`Serving ${rootDir} → http://127.0.0.1:${port}`);
});
