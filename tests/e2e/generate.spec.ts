import { test, expect } from '@playwright/test';
import path from 'path';
import { stat, readFile } from 'fs/promises';
import { PDFDocument } from 'pdf-lib';

const fixturesDir = path.resolve(__dirname, '../fixtures');
const frontAPath = path.join(fixturesDir, 'front_a.png');
const frontBPath = path.join(fixturesDir, 'front_b.jpg');
const backPath = path.join(fixturesDir, 'back.png');
const corruptPath = path.join(fixturesDir, 'corrupt.jpg');

async function expectDownloadSizeGreaterThan(downloadPath: string | null, bytes: number) {
  if (!downloadPath) {
    throw new Error('Download path was null.');
  }
  const info = await stat(downloadPath);
  expect(info.size).toBeGreaterThan(bytes);
}

test('user can generate and download a PDF with mirrored backs', async ({ page }) => {
  await page.goto('/');

  const status = page.locator('#status');
  await expect(status).toContainText('Add at least one front card image');

  await page.locator('#files').setInputFiles([frontAPath, frontBPath]);
  await expect(status).toContainText('Ready to generate 2 front cards.');
  await expect(page.locator('#fileList li')).toHaveCount(2);

  await page.locator('#back').setInputFiles(backPath);

  await expect(page.getByRole('button', { name: /Generate PDF/i })).toBeEnabled();

  await page.getByRole('button', { name: /Generate PDF/i }).click();

  try {
    await expect(status).toContainText('Building your PDF', { timeout: 1_000 });
  } catch {
    // Generation can finish almost instantly on fast machines; that's okay.
  }
  await expect(status).toContainText('PDF ready with 2 cards.', { timeout: 10_000 });

  const previewFrame = page.locator('#pdfContainer');
  await expect(previewFrame).toBeVisible({ timeout: 10_000 });

  const downloadLink = page.locator('#downloadLink');
  await expect(downloadLink).toBeVisible();

  const downloadPromise = page.waitForEvent('download');
  await downloadLink.click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe('cards.pdf');
  await expectDownloadSizeGreaterThan(await download.path(), 700);
});

test('user can generate a PDF without a back image', async ({ page }) => {
  await page.goto('/');

  const status = page.locator('#status');
  await expect(status).toContainText('Add at least one front card image');

  await page.locator('#files').setInputFiles([frontAPath, frontBPath]);
  await expect(status).toContainText('Ready to generate 2 front cards.');

  await page.getByRole('button', { name: /Generate PDF/i }).click();

  try {
    await expect(status).toContainText('Building your PDF', { timeout: 1_000 });
  } catch {}
  await expect(status).toContainText('PDF ready with 2 cards.', { timeout: 10_000 });

  await expect(page.locator('#pdfContainer')).toBeVisible();

  const downloadPromise = page.waitForEvent('download');
  await page.locator('#downloadLink').click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe('cards.pdf');
  await expectDownloadSizeGreaterThan(await download.path(), 500);
});

test('mirror toggle affects back page orientation', async ({ page }) => {
  await page.goto('/');

  const status = page.locator('#status');
  await page.locator('#files').setInputFiles([frontAPath, frontBPath]);
  await page.locator('#back').setInputFiles(backPath);

  // Generate with mirroring enabled (default).
  await page.getByRole('button', { name: /Generate PDF/i }).click();
  await expect(status).toContainText('PDF ready with 2 cards.', { timeout: 10_000 });
  const firstDownloadPromise = page.waitForEvent('download');
  await page.locator('#downloadLink').click();
  const firstDownload = await firstDownloadPromise;
  const mirroredBytes = await readFile(await firstDownload.path());

  // Disable mirror and regenerate.
  await page.locator('#mirrorBack').uncheck();
  await page.getByRole('button', { name: /Generate PDF/i }).click();
  await expect(status).toContainText('PDF ready with 2 cards.', { timeout: 10_000 });
  const secondDownloadPromise = page.waitForEvent('download');
  await page.locator('#downloadLink').click();
  const secondDownload = await secondDownloadPromise;
  const straightBytes = await readFile(await secondDownload.path());

  expect(Buffer.compare(mirroredBytes, straightBytes)).not.toBe(0);
});

test('failed image decode surfaces an error and hides the preview', async ({ page }) => {
  await page.goto('/');

  const status = page.locator('#status');
  const previewFrame = page.locator('#pdfContainer');
  const downloadLink = page.locator('#downloadLink');

  await page.locator('#files').setInputFiles([corruptPath]);
  await page.getByRole('button', { name: /Generate PDF/i }).click();

  await expect(status).toContainText("Failed to decode front image");
  await expect(status).toContainText('corrupt.jpg');
  await expect(status).toHaveClass(/status-error/);
  await expect(previewFrame).toBeHidden();
  await expect(downloadLink).toBeHidden();
});

test('generating many cards spans multiple pages', async ({ page }) => {
  await page.goto('/');

  const status = page.locator('#status');
  const manyFronts = Array.from({ length: 20 }, (_, i) =>
    path.join(fixturesDir, `page_${String(i + 1).padStart(2, '0')}.png`)
  );

  await page.locator('#files').setInputFiles(manyFronts);
  await expect(status).toContainText('Ready to generate 20 front cards.');

  await page.getByRole('button', { name: /Generate PDF/i }).click();
  await expect(status).toContainText('PDF ready with 20 cards.', { timeout: 15_000 });

  const downloadPromise = page.waitForEvent('download');
  await page.locator('#downloadLink').click();
  const download = await downloadPromise;
  const pdfBytes = await readFile(await download.path());
  const pdfDoc = await PDFDocument.load(pdfBytes);
  expect(pdfDoc.getPageCount()).toBeGreaterThanOrEqual(2);
});
