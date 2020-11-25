FROM golang:1.15-alpine AS build

# RUN apk add --no-cache git
WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . ./
# # Unit tests
# RUN CGO_ENABLED=0 go test -v
RUN go build -o app .

##############
FROM alpine
RUN apk add ca-certificates

COPY --from=build /app/app /app
CMD ["/app"]
