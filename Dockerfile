FROM golang:1.18-alpine AS build

WORKDIR /build

COPY go.mod go.sum ./
RUN go mod download

COPY . ./
# # Unit tests
# RUN go test -v
RUN CGO_ENABLED=0 go build -ldflags="-w -s" -o /app .

##############
FROM gcr.io/distroless/static-debian11
USER nobody

COPY --from=build /app /app
CMD ["/app"]