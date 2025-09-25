import logging


class SingletonLogger:
    """
    Custom project singleton logger to centralize config and avoid
    chicken-and-egg import issues.
    """

    _instance = None

    def __new__(cls, name="generate-es-reports"):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance._initialized = False
        return cls._instance

    def __init__(self, name="generate-es-reports"):
        if self._initialized:
            return

        self.logger = logging.getLogger(name)
        self.logger.setLevel(logging.DEBUG)

        formatter = logging.Formatter("%(asctime)s [%(levelname)s] - %(message)s")
        console_handler = logging.StreamHandler()
        console_handler.setFormatter(formatter)
        self.logger.addHandler(console_handler)

        self._initialized = True

    def get_logger(self):
        return self.logger
