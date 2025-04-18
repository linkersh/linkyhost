import winston, { createLogger } from "winston";

const log = createLogger({
  format: winston.format.combine(
    winston.format.colorize(),
    winston.format.prettyPrint(),
    winston.format.simple()
  ),
  levels: winston.config.cli.levels,
  transports: [new winston.transports.Console()],
});

export default log;