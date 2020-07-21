FROM node:alpine
ENV NODE_ENV=production

WORKDIR /app
COPY package.json yarn.lock  ./
RUN yarn install

COPY . ./
CMD ["yarn", "start"]
