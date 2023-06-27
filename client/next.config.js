/** @type {import('next').NextConfig} */
const nextConfig = {
  experiments: {
    asyncWebAssembly: true,
    topLevelAwait: true,
  },
};

module.exports = nextConfig;
