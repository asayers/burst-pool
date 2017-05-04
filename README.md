A thread pool optimised for bursts of activity.

Designed for the following use-case: A single thread produces work which must
then be performed by a pool of worker threads. The work is produced
infrequently, but in bursts. Under normal operation, therefore, the threads in
the pool sleep until some event requires many of them to be suddenly be woken
at once. Those threads perform some work before going back to sleep again.

See the [documentation] for details.

[documentation]: https://docs.rs/burst-pool/
