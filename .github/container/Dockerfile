FROM scratch

ARG TARGETARCH
COPY --chmod=0555 linux/${TARGETARCH}/cron-when /cron-when
ENV TZ=UTC
USER 65532:65532
ENTRYPOINT ["/cron-when"]
