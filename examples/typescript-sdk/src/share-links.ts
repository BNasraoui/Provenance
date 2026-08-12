const SEVEN_DAYS = 7 * 24 * 60 * 60 * 1000;

export function createShareLink(now = new Date()) {
  return {
    expiresAt: new Date(now.getTime() + SEVEN_DAYS),
  };
}
