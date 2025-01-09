import { Box, Container, Flex, Heading, Button, Link as ChakraLink } from '@chakra-ui/react';
import { Link } from 'react-router-dom';

const Navbar = () => {
  return (
    <Box bg="white" boxShadow="sm" position="sticky" top={0} zIndex={1}>
      <Container maxW="container.xl">
        <Flex h={16} alignItems="center" justifyContent="space-between">
          <Link to="/">
            <Heading size="md" color="blue.600">Bonsol Proof Explorer</Heading>
          </Link>
          <Flex gap={4}>
            <Button
              as={Link}
              to="/"
              variant="ghost"
              colorScheme="blue"
            >
              All Requests
            </Button>
            <ChakraLink
              href="https://github.com/your-org/bonsol"
              target="_blank"
              rel="noopener noreferrer"
            >
              <Button variant="ghost" colorScheme="blue">
                Documentation
              </Button>
            </ChakraLink>
          </Flex>
        </Flex>
      </Container>
    </Box>
  );
};

export default Navbar; 
