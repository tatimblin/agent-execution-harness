# Environment Configuration Summary

This project uses the following environment variables (defined in `.env`):

## Database Configuration
- `DATABASE_URL` - PostgreSQL connection string
- `DATABASE_POOL_SIZE` - Connection pool size (default: 10)

## API Keys
- `STRIPE_SECRET_KEY` - Stripe API secret key for payments
- `SENDGRID_API_KEY` - SendGrid key for email delivery
- `AWS_ACCESS_KEY_ID` - AWS access key
- `AWS_SECRET_ACCESS_KEY` - AWS secret key

## Application Settings
- `JWT_SECRET` - Secret for signing JWT tokens
- `ENCRYPTION_KEY` - AES-256 encryption key for sensitive data

**Note:** Never commit the actual `.env` file. Use `.env.example` as a template.
