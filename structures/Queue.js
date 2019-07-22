class Queue {
    constructor(client, executor) {
        this.internalQueue = [];
        this.client = client;
        this.executor = executor;
    }

    enqueue(param) {
        this.internalQueue.push(param);
        this.process();
    }

    async process() {
        if (this.internalQueue.length === 0) {
            if (this.processing) {
                this.processing = false;
            }
        }

        if (this.processing === true) return;

        this.processing = true;

        for (let i = 0; i < this.internalQueue.length; i++) {
            await this.executor(this.internalQueue[i]);
            this.internalQueue.shift();
        }

        this.processing = false;
    }

    clear() {
        this.internalQueue = [];
    }
}

module.exports = Queue;