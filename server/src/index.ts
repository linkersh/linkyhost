//@ts-ignore
BigInt.prototype.toJSON = function () {
  return this.toString();
};

import Elysia from "elysia";
import log from "@/utils/log";
import cors from "@elysiajs/cors";
import { apiRouter } from "@/routes";

new Elysia()
  .use(cors())
  .use(apiRouter())
  .onError(({ error }) => {
    log.error(error);
  })
  .listen(9090, () => {
    log.info("Server is running on http://localhost:9090");
  });
